use crate::{
    Version,
    utils::{
        codec::{CodecError, Decode, Encode},
        signature::Signature,
        vec::CodecVec,
    },
    v1::{LogEntry, SignatureType},
};
use std::io::{Read, Write};

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedCertificateTimestamp {
    sct_version: Version,
    id: [u8; 32],
    extensions: CodecVec<u16>,
    signature: Signature<CertificateTimeStamp>,
}

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CertificateTimeStamp {
    sct_version: Version,
    // SignatureType signature_type = certificate_timestamp;
    timestamp: u64,
    entry: LogEntry,
    extensions: CodecVec<u16>,
}

impl Encode for CertificateTimeStamp {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.sct_version.encode(&mut writer)?;
        SignatureType::CertificateTimeStamp.encode(&mut writer)?;
        self.timestamp.encode(&mut writer)?;
        self.entry.encode(&mut writer)?;
        self.extensions.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for CertificateTimeStamp {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let sct_version = Version::decode(&mut reader)?;
        let signature_type = SignatureType::decode(&mut reader)?;
        match signature_type {
            SignatureType::TreeHash => return Err(CodecError::UnexpectedVariant),
            SignatureType::CertificateTimeStamp => (),
        }
        let timestamp = u64::decode(&mut reader)?;
        let entry = LogEntry::decode(&mut reader)?;
        let extensions = CodecVec::decode(&mut reader)?;

        Ok(Self {
            sct_version,
            timestamp,
            entry,
            extensions,
        })
    }
}
