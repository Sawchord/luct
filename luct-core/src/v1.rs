use crate::utils::{
    codec::{CodecError, Decode, Encode},
    u24::U24,
    vec::CodecVec,
};
pub(crate) use sct::SctList;
use std::io::{Read, Write};
use x509_cert::{
    certificate::{CertificateInner, Rfc5280, TbsCertificateInner},
    der::{Decode as DerDecode, Encode as DerEncode},
};

pub(crate) mod merkle_tree;
pub(crate) mod sct;
pub(crate) mod sth;

pub use merkle_tree::{GetEntriesResponse, LeafHash, MerkleTreeLeaf};
pub use sct::SignedCertificateTimestamp;
pub use sth::SthResponse;

/// See RFC 5246 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SignatureType {
    CertificateTimeStamp,
    TreeHash,
}

impl Encode for SignatureType {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let discriminant = match self {
            SignatureType::CertificateTimeStamp => 0,
            SignatureType::TreeHash => 1,
        };
        Ok(writer.write_all(&[discriminant])?)
    }
}

impl Decode for SignatureType {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = vec![0u8];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(SignatureType::CertificateTimeStamp),
            1 => Ok(SignatureType::TreeHash),
            x => Err(CodecError::UnknownVariant("SignatureType", x as u64)),
        }
    }
}

// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogEntry {
    X509(CertificateInner<Rfc5280>),
    PreCert(PreCert),
}

impl Encode for LogEntry {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        match self {
            LogEntry::X509(cert) => {
                writer.write_all(&[0, 0])?;

                let mut cert_bytes = vec![];
                let _len = cert.encode_to_vec(&mut cert_bytes)?;
                CodecVec::<U24>::from(cert_bytes).encode(&mut writer)?;
            }
            LogEntry::PreCert(pre_cert) => {
                writer.write_all(&[0, 1])?;
                pre_cert.encode(&mut writer)?;
            }
        };

        Ok(())
    }
}

impl Decode for LogEntry {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        let entry = u16::from_be_bytes(buf);

        match entry {
            0 => {
                let cert_bytes = CodecVec::<U24>::decode(&mut reader)?;
                let cert = CertificateInner::<Rfc5280>::from_der(cert_bytes.as_ref())?;
                Ok(LogEntry::X509(cert))
            }
            1 => Ok(LogEntry::PreCert(PreCert::decode(&mut reader)?)),
            x => Err(CodecError::UnknownVariant("LogEntry", x as u64)),
        }
    }
}

// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreCert {
    pub(crate) issuer_key_hash: [u8; 32],
    pub(crate) tbs_certificate: TbsCertificateInner<Rfc5280>,
}

impl Encode for PreCert {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        crate::Encode::encode(&self.issuer_key_hash, &mut writer)?;

        let mut cert_bytes = vec![];
        let _len = self.tbs_certificate.encode_to_vec(&mut cert_bytes)?;
        CodecVec::<U24>::from(cert_bytes).encode(&mut writer)?;

        Ok(())
    }
}

impl Decode for PreCert {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let issuer_key_hash = <[u8; 32] as Decode>::decode(&mut reader)?;
        let cert_bytes = CodecVec::<U24>::decode(&mut reader)?;
        let tbs_certificate = TbsCertificateInner::<Rfc5280>::from_der(cert_bytes.as_ref())?;

        Ok(Self {
            issuer_key_hash,
            tbs_certificate,
        })
    }
}

// TODO: LogEntryChain
