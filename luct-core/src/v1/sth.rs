use std::io::{Read, Write};

use crate::utils::{
    base64::Base64,
    codec::{Codec, CodecError, Decode, Encode},
    signature::Signature,
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
    // TODO:
    // Version version;
    // SignatureType signature_type = tree_hash;
    // uint64 timestamp;
    // uint64 tree_size;
    // opaque sha256_root_hash[32];
}

impl Encode for TreeHeadSignature {
    fn encode(&self, _writer: impl Write) -> Result<(), CodecError> {
        // TODO: Update with fields
        Ok(())
    }
}

impl Decode for TreeHeadSignature {
    fn decode(_reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {})
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
