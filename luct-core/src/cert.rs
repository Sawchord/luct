use crate::{
    utils::codec::{CodecError, Decode},
    v1::{SctList, SignedCertificateTimestamp},
};
use p256::pkcs8::ObjectIdentifier;
use std::io::Cursor;
use thiserror::Error;
use x509_cert::{
    Certificate as Cert,
    der::{Decode as CertDecode, DecodePem, asn1::OctetString},
};

const SCT_V1: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.6.1.4.1.11129.2.4.2");

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

    // TODO: into_precert
    // TODO: is_precert
}

// TODO: Implement Encode and Decode and use it instead
// of the vectors in types

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CertificateError {
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
    use crate::utils::codec::Encode;

    use super::*;

    const CERT_CHAIN_GOOGLE_COM: &str = include_str!("../testdata/google-chain.pem");

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
        let _scts = cert.extract_scts_v1().unwrap();

        // TODO: Check that log id matches the id of CTLog
        // TODO: Validate sct against log
    }
}
