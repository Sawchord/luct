use std::io::{Read, Write};

use crate::{
    Version,
    utils::{
        base64::Base64,
        codec::{Codec, CodecError, Decode, Encode},
        signature::Signature,
    },
    v1::SignatureType,
};
use serde::{Deserialize, Serialize};

/// See RFC 6962 4.3
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SthResponse {
    tree_size: usize,
    // TODO: Use a dedicated timestamp type
    timestamp: u64,
    sha256_root_hash: Base64<Vec<u8>>,
    tree_head_signature: Base64<Codec<Signature<TreeHeadSignature>>>,
}

/// See RFC
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TreeHeadSignature {
    version: Version,
    // SignatureType signature_type = tree_hash;
    timestamp: u64,
    tree_size: u64,
    sha256_root_hash: [u8; 32],
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

#[cfg(test)]
mod test {
    use super::*;

    const ARGON2025H1_STH2806: &str = "{
    \"tree_size\":1425614114,
    \"timestamp\":1751114416696,
    \"sha256_root_hash\":\"LHtW79pwJohJF5Yn/tyozEroOnho4u3JAGn7WeHSR54=\",
    \"tree_head_signature\":\"BAMARzBFAiEAg4w8LlTFKd3KL6lo5Zde9OupHYNN0DDk8U54PenirI4CIHL8ucpkJw5zFLh8UvLA+Zf+f8Ms+tLsVtzHuqnO0qjm\"
    }";

    #[test]
    fn decode_sth() {
        let _sth: SthResponse = serde_json::from_str(ARGON2025H1_STH2806).unwrap();
    }
}
