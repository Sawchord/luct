use crate::utils::{
    codec::{CodecError, Decode, Encode},
    u24::U24,
    vec::CodecVec,
};
use std::io::{Read, Write};

mod merkle_tree;
mod sct;
mod sth;

pub use sct::{SctList, SignedCertificateTimestamp};
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LogEntry {
    X509(CodecVec<U24>),
    PreCert(PreCert),
}

impl Encode for LogEntry {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        match self {
            LogEntry::X509(cert) => {
                writer.write_all(&[0])?;
                cert.encode(&mut writer)?;
            }
            LogEntry::PreCert(pre_cert) => {
                writer.write_all(&[1])?;
                pre_cert.encode(&mut writer)?;
            }
        };

        Ok(())
    }
}

impl Decode for LogEntry {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(LogEntry::X509(CodecVec::decode(&mut reader)?)),
            1 => Ok(LogEntry::PreCert(PreCert::decode(&mut reader)?)),
            x => Err(CodecError::UnknownVariant("LogEntry", x as u64)),
        }
    }
}

// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PreCert {
    issuer_key_hash: [u8; 32],
    tbs_certificate: CodecVec<U24>,
}

impl Encode for PreCert {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.issuer_key_hash.encode(&mut writer)?;
        self.tbs_certificate.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for PreCert {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            issuer_key_hash: <[u8; 32]>::decode(&mut reader)?,
            tbs_certificate: CodecVec::decode(&mut reader)?,
        })
    }
}
