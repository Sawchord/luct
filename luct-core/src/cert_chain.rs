use crate::{
    Certificate, CertificateError,
    cert::{CT_POISON, SCT_V1},
    utils::codec::CodecError,
    v1,
};
use sha2::{Digest, Sha256};
use x509_cert::{Certificate as Cert, der::Encode};
use x509_verify::VerifyingKey;

/// A [`CertificateChain`] chain of trust
///
/// These chains are what gets presented by TLS.
/// They consist of a number of X.509 [`Certificates`](Certificate),
/// from the source to a root of trust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateChain(Vec<Certificate>);

// TODO: Iterator over CertChain

impl From<Vec<Certificate>> for CertificateChain {
    fn from(value: Vec<Certificate>) -> Self {
        Self(value)
    }
}

impl CertificateChain {
    pub fn from_pem_chain(input: &str) -> Result<Self, CertificateError> {
        let chain = Cert::load_pem_chain(input.as_bytes())?;

        // We need at least a chain of depth 2 (root + leaf), since root certs themselves
        // can not be logged in this way
        if chain.len() < 2 {
            return Err(CertificateError::InvalidChain);
        }

        let chain = Self(chain.into_iter().map(Certificate).collect());
        chain.verify_chain()?;
        Ok(chain)
    }

    pub fn verify_chain(&self) -> Result<(), CertificateError> {
        self.verify_chain_inner(None)
    }

    pub fn verify_chain_against_root(&self, root: &Certificate) -> Result<(), CertificateError> {
        self.verify_chain_inner(Some(root))
    }

    fn verify_chain_inner(&self, maybe_root: Option<&Certificate>) -> Result<(), CertificateError> {
        for idx in 1..self.0.len() {
            let key = VerifyingKey::try_from(&self.0[idx].0)?;
            key.verify(&self.0[idx - 1].0)?;
        }

        if let Some(root) = maybe_root {
            let key = VerifyingKey::try_from(&self.0.last().unwrap().0)?;
            key.verify(&root.0)?;
        }

        Ok(())
    }

    pub fn cert(&self) -> &Certificate {
        &self.0[0]
    }

    pub fn root(&self) -> &Certificate {
        self.0.last().unwrap()
    }

    pub(crate) fn as_log_entry_v1(
        &self,
        as_precert: bool,
    ) -> Result<v1::LogEntry, CertificateError> {
        if !as_precert {
            return Ok(v1::LogEntry::X509(self.cert().0.clone()));
        }

        let mut subject_public_key_bytes = vec![];
        let mut tbs_certificate = self.cert().0.tbs_certificate.clone();

        // Get the hash of the issuers subject public key info
        self.0[1]
            .0
            .tbs_certificate
            .subject_public_key_info
            .encode_to_vec(&mut subject_public_key_bytes)?;
        let issuer_key_hash: [u8; 32] = Sha256::digest(&subject_public_key_bytes).into();

        // TODO: Change the issuer, if a special precert signing certificate is being used

        tbs_certificate.extensions = tbs_certificate.extensions.map(|extensions| {
            extensions
                .into_iter()
                // NOTE: We need to remove all SCT and POISON extensions
                .filter(|extension| extension.extn_id != SCT_V1 && extension.extn_id != CT_POISON)
                .collect::<Vec<_>>()
        });

        Ok(v1::LogEntry::PreCert(v1::PreCert {
            issuer_key_hash,
            tbs_certificate,
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
                log_entry: self.as_log_entry_v1(as_precert).map_err(|err| match err {
                    CertificateError::DerParseError(err) => CodecError::DerError(err),
                    CertificateError::CodecError(err) => err,
                    _ => unreachable!(),
                })?,
                extensions: sct.extensions.clone(),
            }),
        })
    }
}
