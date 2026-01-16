use crate::{
    CertificateChain, CertificateError, CtLog, Version,
    signature::{Signature, SignatureValidationError},
    store::Hashable,
    tree::HashOutput,
    utils::{
        append_vec::{SizedAppendVec, SizedVal},
        codec::{CodecError, Decode, Encode},
    },
    v1::{
        LogEntry, LogId, SignatureType,
        extension::{CtExtensions, LeafIndex},
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};

impl CtLog {
    pub fn validate_sct_v1(
        &self,
        cert: &CertificateChain,
        sct: &SignedCertificateTimestamp,
        as_precert: bool,
    ) -> Result<(), SignatureValidationError> {
        let timestamp = CertificateTimeStamp {
            sct_version: Version::V1,
            timestamp: sct.timestamp,
            entry: cert.as_log_entry_v1(as_precert).map_err(|err| match err {
                CertificateError::CodecError(err) => SignatureValidationError::CodecError(err),
                _ => unreachable!(),
            })?,
            extensions: sct.extensions.clone(),
        };

        sct.signature.validate(&timestamp, &self.config.key)
    }
}

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SctList(SizedAppendVec<SignedCertificateTimestamp>);

impl SctList {
    #[allow(dead_code)]
    pub fn new(scts: Vec<SignedCertificateTimestamp>) -> Self {
        Self(SizedAppendVec::from(scts))
    }

    pub fn into_inner(self) -> Vec<SignedCertificateTimestamp> {
        self.0.into()
    }
}

impl Encode for SctList {
    fn encode(&self, writer: impl Write) -> Result<(), CodecError> {
        self.0.encode(writer)
    }
}

impl Decode for SctList {
    fn decode(reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self(SizedAppendVec::decode(reader)?))
    }
}

/// A signed certificate timestamp of version 1.
///
/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedCertificateTimestamp {
    pub(crate) sct_version: Version,
    pub(crate) id: LogId,
    pub(crate) timestamp: u64,
    pub(crate) extensions: CtExtensions,
    pub(crate) signature: Signature<CertificateTimeStamp>,
}

impl SignedCertificateTimestamp {
    pub fn log_id(&self) -> crate::LogId {
        crate::LogId::V1(self.id.clone())
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn leaf_index(&self) -> Option<LeafIndex> {
        self.extensions.leaf_index()
    }
}

impl Encode for SignedCertificateTimestamp {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.sct_version.encode(&mut writer)?;
        self.id.encode(&mut writer)?;
        self.timestamp.encode(&mut writer)?;
        self.extensions.encode(&mut writer)?;
        self.signature.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for SignedCertificateTimestamp {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            sct_version: Version::decode(&mut reader)?,
            id: LogId::decode(&mut reader)?,
            timestamp: u64::decode(&mut reader)?,
            extensions: CtExtensions::decode(&mut reader)?,
            signature: Signature::decode(&mut reader)?,
        })
    }
}

impl Hashable for SignedCertificateTimestamp {
    fn hash(&self) -> HashOutput {
        let mut bytes = Cursor::new(vec![]);
        self.encode(&mut bytes).unwrap();
        Sha256::digest(bytes.into_inner()).into()
    }
}

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CertificateTimeStamp {
    sct_version: Version,
    // SignatureType signature_type = certificate_timestamp;
    timestamp: u64,
    entry: LogEntry,
    extensions: CtExtensions,
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
        let extensions = CtExtensions::decode(&mut reader)?;

        Ok(Self {
            sct_version,
            timestamp,
            entry,
            extensions,
        })
    }
}
