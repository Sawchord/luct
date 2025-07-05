use crate::{
    utils::codec::{CodecError, Decode},
    v1::{LogEntry, PreCert, SctList, SignedCertificateTimestamp},
};
use p256::pkcs8::ObjectIdentifier;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use thiserror::Error;
use x509_cert::{
    Certificate as Cert,
    der::{Decode as CertDecode, DecodePem, Encode, asn1::OctetString},
};

const SCT_V1: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.4.1.11129.2.4.2");
const CT_POISON: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.4.1.11129.2.4.3");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateChain(Vec<Certificate>);

impl CertificateChain {
    pub fn from_pem_chain(input: &str) -> Result<Self, CertificateError> {
        let chain = Cert::load_pem_chain(input.as_bytes())?;

        // We need at least a chain of depth 2 (root + leaf), since root certs themselves
        // can not be logged in this way
        if chain.len() < 2 {
            return Err(CertificateError::InvalidChain);
        }

        // TODO: Validate the cert against the actual certificates

        Ok(Self(chain.into_iter().map(Certificate).collect()))
    }

    pub fn cert(&self) -> &Certificate {
        &self.0[0]
    }

    pub fn root(&self) -> &Certificate {
        self.0.last().unwrap()
    }

    pub(crate) fn as_log_entry_v1(&self) -> Result<LogEntry, CertificateError> {
        Ok(LogEntry::X509(self.cert().0.clone()))
    }

    pub(crate) fn as_precert_entry_v1(&self) -> Result<LogEntry, CertificateError> {
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

        Ok(LogEntry::PreCert(PreCert {
            issuer_key_hash,
            tbs_certificate,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate(Cert);

impl Certificate {
    pub fn from_pem(input: &str) -> Result<Self, CertificateError> {
        Ok(Self(Cert::from_pem(input.as_bytes())?))
    }

    pub fn extract_scts_v1(&self) -> Result<Vec<SignedCertificateTimestamp>, CertificateError> {
        let Some(extensions) = &self.0.tbs_certificate.extensions else {
            return Ok(vec![]);
        };

        let sct_lists = extensions
            .iter()
            .filter(|extension| extension.extn_id == SCT_V1)
            .map(|sct| &sct.extn_value)
            .map(|sct| {
                let sct = OctetString::from_der(sct.as_bytes()).unwrap();
                let mut reader = Cursor::new(sct.as_bytes());
                SctList::decode(&mut reader)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let scts = sct_lists
            .into_iter()
            .flat_map(|list| list.into_inner())
            .collect();

        Ok(scts)
    }

    pub fn is_precert(&self) -> Result<bool, CertificateError> {
        let Some(extensions) = &self.0.tbs_certificate.extensions else {
            return Ok(false);
        };

        let scts = extensions
            .iter()
            .filter(|extension| extension.extn_id == SCT_V1)
            .count();

        let poisons = extensions
            .iter()
            .filter(|extension| extension.extn_id == CT_POISON && extension.critical)
            .filter(|extension| extension.extn_value.as_bytes() == [0x05, 0x00])
            .count();

        match (poisons, scts) {
            (1, 0) => Ok(true),
            (0, _) => Ok(false),
            _ => Err(CertificateError::InvalidPreCert),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CertificateError {
    #[error("A precert can't have SCTs or more than one poison value")]
    InvalidPreCert,

    #[error("The certificate chain is malformed")]
    InvalidChain,

    #[error("Failed to parse a DER encoded certificate: {0}")]
    DerParseError(#[from] x509_cert::der::Error),

    #[error("Failed to decode a value {0}")]
    CodecError(#[from] CodecError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::get_log_argon2025h2, utils::codec::Encode};

    const CERT_CHAIN_GOOGLE_COM: &str = include_str!("../testdata/google-chain.pem");
    const CERT_GOOGLE_COM: &str = include_str!("../testdata/google-cert.pem");
    const PRE_CERT_GOOGLE_COM: &str = include_str!("../testdata/google-precert.pem");

    #[test]
    fn sct_list_codec_rountrip() {
        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let mut writer = Cursor::new(vec![]);
        SctList::new(scts.clone()).encode(&mut writer).unwrap();
        let scts2 = SctList::decode(Cursor::new(writer.into_inner()))
            .unwrap()
            .into_inner();

        assert_eq!(scts, scts2)
    }

    #[test]
    fn validate_scts() {
        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let log = get_log_argon2025h2();
        assert_eq!(log.log_id_v1(), scts[0].log_id());

        log.validate_sct_as_precert_v1(&cert, &scts[0]).unwrap();
    }

    #[test]
    fn precert_transformation() {
        let cert1 = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let cert2 = Certificate::from_pem(CERT_GOOGLE_COM).unwrap();

        assert_eq!(cert1.cert(), &cert2);
        assert!(!cert1.cert().is_precert().unwrap());

        let precert = Certificate::from_pem(PRE_CERT_GOOGLE_COM).unwrap();
        assert!(precert.is_precert().unwrap());

        // assert_eq!(
        //     cert1.cert().as_precert_entry_v1(),
        //     precert.as_precert_entry_v1()
        // );
    }
}
