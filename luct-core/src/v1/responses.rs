//! The response structures of a v1 log.
//!
//! These structures correspond to the ones described in RFC 6962 Section 4.
//! They can be parsed using `serde_json`, and then be transformed into other structures to be validated.

use crate::{
    signature::Signature,
    utils::{base64::Base64, codec::Codec},
    v1::{MerkleTreeLeaf, sth::TreeHeadSignature},
};
use serde::{Deserialize, Serialize};

/// Response returned by call to `/ct/v1/get-sth`
///
/// See RFC 6962 4.3
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GetSthResponse {
    pub(crate) tree_size: u64,
    pub(crate) timestamp: u64,
    pub(crate) sha256_root_hash: Base64<Vec<u8>>,
    pub(crate) tree_head_signature: Base64<Codec<Signature<TreeHeadSignature>>>,
}

/// Response returned by call to `/ct/v1/get-sth-consistency`
///
/// See RFC 6962 4.4
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GetSthConsistencyResponse {
    pub(crate) consistency: Vec<Base64<Vec<u8>>>,
}

/// Response returned by call to `/ct/v1/get-proof-by-hash`
///
/// See RFC 6962 4.5
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GetProofByHashResponse {
    pub(crate) leaf_index: u64,
    pub(crate) audit_path: Vec<Base64<Vec<u8>>>,
}

/// Response returned by call to `/ct/v1/get-entries`
///
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

// TODO: GetRoots support
