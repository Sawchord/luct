use serde::{Deserialize, Serialize};

use crate::{
    CtLog, Version,
    signature::{Signature, SignatureValidationError},
    tree::HashOutput,
    utils::codec::{CodecError, Decode, Encode},
    v1::{SignatureType, responses::GetSthResponse},
};
use std::io::{Read, Write};

impl CtLog {
    pub fn validate_sth_v1(&self, sth: &SignedTreeHead) -> Result<(), SignatureValidationError> {
        let tree_head_tbs = TreeHeadSignature::from(sth);
        sth.tree_head_signature
            .validate(&tree_head_tbs, &self.config.key)
    }
}

/// Response returned by call to `/ct/v1/get-sth`
///
/// See RFC 6962 4.3
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SignedTreeHead {
    pub(crate) tree_size: u64,
    pub(crate) timestamp: u64,
    pub(crate) sha256_root_hash: HashOutput,
    pub(crate) tree_head_signature: Signature<TreeHeadSignature>,
}

impl SignedTreeHead {
    pub fn tree_size(&self) -> u64 {
        self.tree_size
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl TryFrom<GetSthResponse> for SignedTreeHead {
    type Error = ();

    fn try_from(value: GetSthResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            tree_size: value.tree_size,
            timestamp: value.timestamp,
            sha256_root_hash: value.sha256_root_hash.0.try_into().map_err(|_| ())?,
            tree_head_signature: value.tree_head_signature.0.0,
        })
    }
}

/// See RFC 6962 3.5
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TreeHeadSignature {
    pub(crate) version: Version,
    // SignatureType signature_type = tree_hash;
    pub(crate) timestamp: u64,
    pub(crate) tree_size: u64,
    pub(crate) sha256_root_hash: [u8; 32],
}

impl Encode for TreeHeadSignature {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.version.encode(&mut writer)?;
        SignatureType::TreeHash.encode(&mut writer)?;
        self.timestamp.encode(&mut writer)?;
        self.tree_size.encode(&mut writer)?;
        self.sha256_root_hash.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for TreeHeadSignature {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let version = Version::decode(&mut reader)?;
        let signature_type = SignatureType::decode(&mut reader)?;
        match signature_type {
            SignatureType::CertificateTimeStamp => return Err(CodecError::UnexpectedVariant),
            SignatureType::TreeHash => (),
        }
        let timestamp = u64::decode(&mut reader)?;
        let tree_size = u64::decode(&mut reader)?;
        let sha256_root_hash = <[u8; 32]>::decode(&mut reader)?;

        Ok(Self {
            version,
            timestamp,
            tree_size,
            sha256_root_hash,
        })
    }
}

impl From<&SignedTreeHead> for TreeHeadSignature {
    fn from(value: &SignedTreeHead) -> Self {
        Self {
            version: Version::V1,
            timestamp: value.timestamp,
            tree_size: value.tree_size,
            sha256_root_hash: value.sha256_root_hash,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::{ARGON2025H1_STH2806, get_log_argon2025h1};

    #[test]
    fn sth_codec_roundtrip() {
        let sth: GetSthResponse = serde_json::from_str(ARGON2025H1_STH2806).unwrap();
        let sth_bytes = serde_json::to_string(&sth).unwrap();
        let sth2: GetSthResponse = serde_json::from_str(&sth_bytes).unwrap();
        assert_eq!(sth, sth2);
    }

    #[test]
    fn validate_sth() {
        let log = get_log_argon2025h1();
        let sth: GetSthResponse = serde_json::from_str(ARGON2025H1_STH2806).unwrap();
        log.validate_sth_v1(&sth.try_into().unwrap()).unwrap();
    }
}
