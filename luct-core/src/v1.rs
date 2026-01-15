use crate::utils::{
    codec::{CodecError, Decode, Encode},
    codec_vec::CodecVec,
    hex_with_colons,
    u24::U24,
};
pub(crate) use sct::SctList;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    io::{Read, Write},
};
use x509_cert::{
    certificate::{CertificateInner, Rfc5280, TbsCertificateInner},
    der::{Decode as DerDecode, Encode as DerEncode},
};

pub(crate) mod extension;
pub(crate) mod proof;
pub mod responses;
pub(crate) mod roots;
pub(crate) mod sct;
pub(crate) mod sth;
pub(crate) mod tree;

pub use sct::SignedCertificateTimestamp;
pub use sth::SignedTreeHead;
pub use tree::MerkleTreeLeaf;

// TODO(Submission support): Requests and responses for submission

// TODO: LogEntryChain type

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogId(pub(crate) [u8; 32]);

impl Display for LogId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex_with_colons(&self.0))
    }
}

impl Encode for LogId {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        Ok(writer.write_all(&self.0)?)
    }
}

impl Decode for LogId {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;

        Ok(Self(buf))
    }
}

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
pub(crate) enum LogEntry {
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

/// A [`PreCert`] contains a X.509 `TbsCertificate` as well as a hash of the issuer key.
///
/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreCert {
    pub(crate) issuer_key_hash: [u8; 32],
    pub(crate) tbs_certificate: TbsCertificateInner<Rfc5280>,
}

impl Encode for PreCert {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        Encode::encode(&self.issuer_key_hash, &mut writer)?;

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
