use crate::{
    Certificate, CertificateError,
    cert::{CT_POISON, SCT_V1},
    utils::codec::CodecError,
    v1,
};
use itertools::Itertools;
use sha2::{Digest, Sha256};
use std::ops::Index;
use x509_cert::{
    Certificate as Cert,
    builder::{
        Builder, CertificateBuilder,
        profile::cabf::{
            self,
            tls::{CertificateType, DomainValidated, Subscriber},
        },
    },
    der::{Decode, Encode},
};

/// A [`CertificateChain`] chain of trust
///
/// These chains are what gets presented by TLS.
/// They consist of a number of X.509 [`Certificates`](Certificate),
/// from the source to a root of trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateChain(Vec<Certificate>);

impl From<Vec<Certificate>> for CertificateChain {
    fn from(value: Vec<Certificate>) -> Self {
        Self(value)
    }
}

impl CertificateChain {
    pub fn from_pem_chain(input: &str) -> Result<Self, CertificateError> {
        let chain = Cert::load_pem_chain(input.as_bytes()).map_err(CodecError::DerError)?;

        // We need at least a chain of depth 2 (root + leaf), since root certs themselves
        // can not be logged in this way
        if chain.len() < 2 {
            return Err(CertificateError::InvalidChain);
        }

        let chain = Self(chain.into_iter().map(Certificate).collect());
        Ok(chain)
    }

    pub fn as_pem_chain(&self) -> String {
        self.0.iter().map(|cert| cert.as_pem()).join("")
    }

    pub fn from_der_chain(input: &[Vec<u8>]) -> Result<Self, CertificateError> {
        let chain = input
            .iter()
            .map(|bytes| Cert::from_der(bytes))
            .collect::<Result<Vec<_>, _>>()
            .map_err(CodecError::DerError)?;

        // We need at least a chain of depth 2 (root + leaf), since root certs themselves
        // can not be logged in this way
        if chain.len() < 2 {
            return Err(CertificateError::InvalidChain);
        }

        let chain = Self(chain.into_iter().map(Certificate).collect());
        Ok(chain)
    }

    pub fn cert(&self) -> &Certificate {
        &self.0[0]
    }

    pub fn root(&self) -> &Certificate {
        self.0.last().unwrap()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn as_log_entry_v1(&self, as_precert: bool) -> Result<v1::LogEntry, CodecError> {
        if !as_precert {
            return Ok(v1::LogEntry::X509(self.cert().0.clone()));
        }

        // Get the hash of the issuers subject public key info
        let mut subject_public_key_bytes = vec![];
        // TODO: Support precert signing certificate
        // If the parrent of the precert is a precert signing certificate,
        // the issuer should be the subject public key info of that certificates parent
        self.0[1]
            .0
            .tbs_certificate()
            .subject_public_key_info()
            .encode_to_vec(&mut subject_public_key_bytes)
            .map_err(CodecError::DerError)?;
        let issuer_key_hash: [u8; 32] = Sha256::digest(&subject_public_key_bytes).into();

        let tbs_certificate = self.cert().0.tbs_certificate().clone();

        let mut builder = CertificateBuilder::new(
            // Subscriber {
            //     certificate_type: CertificateType::DomainValidated(
            //         cabf::Root::new(false, tbs_certificate.subject().clone()).unwrap(),
            //     ),
            //     issuer: tbs_certificate.issuer().clone(),
            //     client_auth: false,
            // },
            cabf::Root::new(false, tbs_certificate.issuer().clone()).unwrap(),
            tbs_certificate.serial_number().clone(),
            *tbs_certificate.validity(),
            tbs_certificate.subject_public_key_info().clone(),
        )
        .unwrap();

        if let Some(extensions) = tbs_certificate.extensions() {
            extensions
                .iter()
                // NOTE: We need to remove all SCT and POISON extensions
                .filter(|extension| extension.extn_id != SCT_V1 && extension.extn_id != CT_POISON)
                .try_for_each(|extension| builder.add_extension(extension.clone()))?;
        }

        struct DummySigner;
        impl KeyPair for DummySigner {}

        let new_tbs_certificate = builder
            .assemble(self.0[1].0.signature().clone(), &DummySigner)
            .unwrap();

        Ok(v1::LogEntry::PreCert(v1::PreCert {
            issuer_key_hash,
            tbs_certificate: new_tbs_certificate.tbs_certificate().clone(),
        }))
    }

    /// Return the [leaf](v1::MerkleTreeLeaf) of the [SCT](v1::SignedCertificateTimestamp)
    ///
    /// # Arguments
    /// -`sct`: The [`v1::SignedCertificateTimestamp`] for which the [leaf](v1::MerkleTreeLeaf) should be generated
    /// -`as_precert`: Whether the [leaf](v1::MerkleTreeLeaf) should contain a precert entry or the certificate itself
    ///
    /// # Note:
    /// If the [SCT](v1::SignedCertificateTimestamp) was obtained by extracting it out of the [`Certificate`] itself
    /// via [`Certificate::extract_scts_v1`], then the corresponding leaf must be a precertificate and `is_precert` should
    /// be set to true.
    pub fn as_leaf_v1(
        &self,
        sct: &v1::SignedCertificateTimestamp,
        as_precert: bool,
    ) -> Result<v1::MerkleTreeLeaf, CodecError> {
        Ok(v1::MerkleTreeLeaf {
            version: sct.sct_version.clone(),
            leaf: v1::tree::Leaf::TimestampedEntry(v1::tree::TimestampedEntry {
                timestamp: sct.timestamp,
                log_entry: self.as_log_entry_v1(as_precert)?,
                extensions: sct.extensions.clone(),
            }),
        })
    }
}

impl Index<usize> for CertificateChain {
    type Output = Certificate;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
