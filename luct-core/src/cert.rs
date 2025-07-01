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
    ext::Extension,
};

const SCT_V1: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.4.1.11129.2.4.2");
const CT_POISON: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.4.1.11129.2.4.3");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate(Cert);

impl Certificate {
    pub fn from_pem(input: &str) -> Result<Self, CertificateError> {
        Ok(Self(Cert::from_pem(input.as_bytes())?))
    }

    pub fn from_validated_pem_chain(
        input: &str,
        _roots: &[Certificate],
    ) -> Result<Self, CertificateError> {
        let chain = Cert::load_pem_chain(input.as_bytes())?;

        // TODO: Validate the cert against the actual certificates

        Ok(Self(chain[0].clone()))
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

    pub fn into_log_entry_v1(self) -> Result<LogEntry, CertificateError> {
        Ok(LogEntry::X509(self.0))
    }

    pub fn into_precert_entry_v1(self) -> Result<LogEntry, CertificateError> {
        let mut subject_public_key_bytes = vec![];
        let mut tbs_certificate = self.0.tbs_certificate;

        tbs_certificate
            .subject_public_key_info
            .encode_to_vec(&mut subject_public_key_bytes)?;
        let issuer_key_hash: [u8; 32] = Sha256::digest(&subject_public_key_bytes).into();

        let poison = Extension {
            extn_id: CT_POISON,
            critical: true,
            extn_value: OctetString::new(vec![0x05, 0x00]).unwrap(),
        };

        let extensions = if let Some(extensions) = tbs_certificate.extensions {
            let mut extensions = extensions
                .into_iter()
                .filter(|extension| extension.extn_id != SCT_V1 && extension.extn_id != CT_POISON)
                .collect::<Vec<_>>();
            extensions.push(poison);
            extensions
        } else {
            vec![poison]
        };

        tbs_certificate.extensions = Some(extensions);

        Ok(LogEntry::PreCert(PreCert {
            issuer_key_hash,
            tbs_certificate,
        }))
    }

    // TODO: into_precert

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

    #[error("Failed to parse a DER encoded certificate: {0}")]
    DerParseError(x509_cert::der::ErrorKind),

    #[error("Failed to decode a value {0}")]
    CodecError(#[from] CodecError),
}

impl From<x509_cert::der::Error> for CertificateError {
    fn from(value: x509_cert::der::Error) -> Self {
        Self::DerParseError(value.kind())
    }
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
        let cert = Certificate::from_validated_pem_chain(CERT_CHAIN_GOOGLE_COM, &[]).unwrap();
        let scts = cert.extract_scts_v1().unwrap();

        let mut writer = Cursor::new(vec![]);
        SctList::new(scts.clone()).encode(&mut writer).unwrap();
        let scts2 = SctList::decode(Cursor::new(writer.into_inner()))
            .unwrap()
            .into_inner();

        assert_eq!(scts, scts2)
    }

    #[test]
    fn validate_google_scts() {
        let cert = Certificate::from_validated_pem_chain(CERT_CHAIN_GOOGLE_COM, &[]).unwrap();
        let scts = cert.extract_scts_v1().unwrap();

        let log = get_log_argon2025h2();
        assert_eq!(log.log_id_v1(), scts[0].log_id());

        // TODO: Validate sct against log
    }

    #[test]
    fn precert_transformation() {
        let cert1 = Certificate::from_validated_pem_chain(CERT_CHAIN_GOOGLE_COM, &[]).unwrap();
        let cert2 = Certificate::from_pem(CERT_GOOGLE_COM).unwrap();

        assert_eq!(cert1, cert2);
        assert!(!cert1.is_precert().unwrap());

        let precert = Certificate::from_pem(PRE_CERT_GOOGLE_COM).unwrap();
        assert!(precert.is_precert().unwrap());

        assert_eq!(
            cert1.into_precert_entry_v1(),
            precert.into_precert_entry_v1()
        );
    }
}
