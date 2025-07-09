use crate::{
    CtLog, Version,
    utils::{
        codec::{CodecError, Decode, Encode},
        signature::SignatureValidationError,
    },
    v1::{SignatureType, responses::GetSthResponse},
};
use std::io::{Read, Write};

impl CtLog {
    pub fn validate_sth_v1(&self, sth: &GetSthResponse) -> Result<(), SignatureValidationError> {
        let tree_head_tbs = TreeHeadSignature::try_from(sth)
            .map_err(|_| SignatureValidationError::MalformedSignature)?;

        sth.tree_head_signature
            .validate(&tree_head_tbs, &self.config.key)
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

impl TryFrom<&GetSthResponse> for TreeHeadSignature {
    type Error = ();

    fn try_from(value: &GetSthResponse) -> Result<Self, Self::Error> {
        let sha256_root_hash: [u8; 32] =
            value.sha256_root_hash.as_ref().try_into().map_err(|_| ())?;

        Ok(Self {
            version: Version::V1,
            timestamp: value.timestamp,
            tree_size: value.tree_size,
            sha256_root_hash,
        })
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
        log.validate_sth_v1(&sth).unwrap();
    }
}
