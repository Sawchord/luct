use crate::{
    utils::{base64::Base64, codec::Codec, signature::Signature},
    v1::{MerkleTreeLeaf, sth::TreeHeadSignature},
};
use serde::{Deserialize, Serialize};

/// See RFC 6962 4.3
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SthResponse {
    pub(crate) tree_size: u64,
    pub(crate) timestamp: u64,
    pub(crate) sha256_root_hash: Base64<Vec<u8>>,
    pub(crate) tree_head_signature: Base64<Codec<Signature<TreeHeadSignature>>>,
}

/// See RFC 6962 4.4
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GetSthConsistencyResponse {
    pub(crate) consistency: Vec<Base64<Vec<u8>>>,
}

/// See RFC 6962 4.5
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GetProofByHashResponse {
    pub(crate) leaf_index: u64,
    pub(crate) audit_path: Vec<Base64<Vec<u8>>>,
}

/// See RFC 6962 4.6
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetEntriesResponse {
    pub(crate) entries: Vec<GetEntriesData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GetEntriesData {
    pub(crate) leaf_input: Base64<Codec<MerkleTreeLeaf>>,
    pub(crate) extra_data: Base64<Vec<u8>>,
}
