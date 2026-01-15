use crate::{
    CheckSeverity, Severity,
    utils::{
        codec::{CodecError, Decode},
        hex_with_colons,
    },
    v1,
};
use p256::pkcs8::ObjectIdentifier;
use sha2::{Digest, Sha256};
use std::{
    fmt::{self, Display},
    io::Cursor,
};
use thiserror::Error;
use x509_cert::{
    Certificate as Cert,
    der::{Decode as CertDecode, DecodePem, Encode as CertEncode, asn1::OctetString},
    ext::pkix::{AuthorityKeyIdentifier, SubjectKeyIdentifier},
};

pub(crate) const SCT_V1: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.4.1.11129.2.4.2");
pub(crate) const CT_POISON: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.3.6.1.4.1.11129.2.4.3");

pub(crate) const SUBJECT_KEY_ID: ObjectIdentifier =
    const_oid::db::rfc5280::ID_CE_SUBJECT_KEY_IDENTIFIER;
pub(crate) const AUTH_KEY_ID: ObjectIdentifier =
    const_oid::db::rfc5280::ID_CE_AUTHORITY_KEY_IDENTIFIER;

/// A X.509 certificate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate(pub(crate) Cert);

impl Certificate {
    /// Parse a PEM decoded string into a [`Certificate`]
    pub fn from_pem(input: &str) -> Result<Self, CertificateError> {
        Ok(Self(
            Cert::from_pem(input.as_bytes()).map_err(CodecError::DerError)?,
        ))
    }

    /// Parse a DER decoded string into a [`Certificate`]
    pub fn from_der(input: &[u8]) -> Result<Self, CertificateError> {
        Ok(Self(Cert::from_der(input).map_err(CodecError::DerError)?))
    }

    /// Extract the [SCTs](v1::SignedCertificateTimestamp) embedded into this [`Certificate`]
    pub fn extract_scts_v1(&self) -> Result<Vec<v1::SignedCertificateTimestamp>, CertificateError> {
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
                v1::SctList::decode(&mut reader)
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

    pub fn fingerprint_sha256(&self) -> Fingerprint {
        let mut cert_bytes = vec![];
        self.0.encode_to_vec(&mut cert_bytes).unwrap();

        let hash: [u8; 32] = Sha256::digest(&cert_bytes).into();
        Fingerprint(hash)
    }

    pub fn get_subject_key_info(&self) -> Option<Vec<u8>> {
        let Some(extensions) = &self.0.tbs_certificate.extensions else {
            return None;
        };

        extensions
            .iter()
            .find(|extension| extension.extn_id == SUBJECT_KEY_ID)
            .and_then(|extension| {
                SubjectKeyIdentifier::from_der(extension.extn_value.as_bytes()).ok()
            })
            .map(|key_id| key_id.0.as_bytes().to_vec())
    }

    pub fn get_authority_key_info(&self) -> Option<Vec<u8>> {
        let Some(extensions) = &self.0.tbs_certificate.extensions else {
            return None;
        };

        extensions
            .iter()
            .find(|extension| extension.extn_id == AUTH_KEY_ID)
            .and_then(|extension| {
                AuthorityKeyIdentifier::from_der(extension.extn_value.as_bytes()).ok()
            })
            .and_then(|key_id| key_id.key_identifier)
            .map(|key_id| key_id.as_bytes().to_vec())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fingerprint(pub [u8; 32]);

impl Display for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex_with_colons(&self.0))
    }
}

/// Error returned when parsing a [`Certificate`] or [`CertificateChain`]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CertificateError {
    #[error("A precert can't have SCTs or more than one poison value")]
    InvalidPreCert,

    #[error("The certificate chain is malformed")]
    InvalidChain,

    #[error("Failed to decode a value: {0}")]
    CodecError(#[from] CodecError),

    #[error("Failed to verify certificate: {0}")]
    VerificationError(x509_verify::Error),
}

impl CheckSeverity for CertificateError {
    fn severity(&self) -> Severity {
        match self {
            CertificateError::InvalidPreCert => Severity::Unsafe,
            CertificateError::InvalidChain => Severity::Unsafe,
            CertificateError::CodecError(codec_error) => codec_error.severity(),
            CertificateError::VerificationError(_) => Severity::Unsafe,
        }
    }
}

impl From<x509_verify::Error> for CertificateError {
    fn from(value: x509_verify::Error) -> Self {
        Self::VerificationError(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CertificateChain,
        tests::{CERT_CHAIN_GOOGLE_COM, get_log_argon2025h2},
        utils::codec::Encode,
    };

    const CERT_GOOGLE_COM: &str = include_str!("../../testdata/google-cert.pem");
    const PRE_CERT_GOOGLE_COM: &str = include_str!("../../testdata/google-precert.pem");
    const GOOGLE_COM_FINGERPRINT: &str = "4B:4F:46:F8:E1:78:B4:08:F9:A7:AF:2B:CE:31:0A:6A:9F:BD:59:37:BD:F8:5B:C5:9B:45:D6:3C:81:61:73:67";

    #[test]
    fn sct_list_codec_rountrip() {
        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        cert.verify_chain().unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let mut writer = Cursor::new(vec![]);
        v1::SctList::new(scts.clone()).encode(&mut writer).unwrap();
        let scts2 = v1::SctList::decode(Cursor::new(writer.into_inner()))
            .unwrap()
            .into_inner();

        assert_eq!(scts, scts2)
    }

    #[test]
    fn validate_scts() {
        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        cert.verify_chain().unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let log = get_log_argon2025h2();
        assert_eq!(log.log_id(), &scts[0].log_id());

        log.validate_sct_v1(&cert, &scts[0], true).unwrap();
    }

    #[test]
    fn precert_transformation() {
        let cert1 = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        cert1.verify_chain().unwrap();
        let cert2 = Certificate::from_pem(CERT_GOOGLE_COM).unwrap();

        assert_eq!(cert1.cert(), &cert2);
        assert!(!cert1.cert().is_precert().unwrap());

        let precert = Certificate::from_pem(PRE_CERT_GOOGLE_COM).unwrap();
        assert!(precert.is_precert().unwrap());
    }

    #[test]
    fn fingerprint() {
        let cert = Certificate::from_pem(CERT_GOOGLE_COM).unwrap();
        let fp = cert.fingerprint_sha256();
        assert_eq!(format!("{fp}"), GOOGLE_COM_FINGERPRINT);
    }
}
