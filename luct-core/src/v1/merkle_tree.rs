use crate::{
    CtLog, Version,
    cert::{CertificateChain, CertificateError},
    utils::{
        base64::Base64,
        codec::{Codec, CodecError, Decode, Encode},
        vec::CodecVec,
    },
    v1::{LogEntry, SignedCertificateTimestamp},
};
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};

impl CtLog {
    pub fn as_precert_leaf(
        cert: &CertificateChain,
        sct: &SignedCertificateTimestamp,
    ) -> Result<MerkleTreeLeaf, CodecError> {
        Ok(MerkleTreeLeaf {
            version: sct.sct_version.clone(),
            leaf: Leaf::TimestampedEntry(TimestampedEntry {
                timestamp: sct.timestamp,
                log_entry: cert.as_precert_entry_v1().map_err(|err| match err {
                    CertificateError::DerParseError(err) => CodecError::DerError(err),
                    CertificateError::CodecError(err) => err,
                    _ => unreachable!(),
                })?,
                extensions: sct.extensions.clone(),
            }),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetEntriesResponse {
    entries: Vec<GetEntriesData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetEntriesData {
    leaf_input: Base64<Codec<MerkleTreeLeaf>>,
    extra_data: Base64<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafHash([u8; 32]);

impl LeafHash {
    pub fn base64(&self) -> String {
        BASE64_STANDARD.encode(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleTreeLeaf {
    version: Version,
    leaf: Leaf,
}

impl MerkleTreeLeaf {
    pub fn hash(&self) -> Result<LeafHash, CodecError> {
        let mut bytes = Cursor::new(vec![]);
        self.encode(&mut bytes)?;

        let hash: [u8; 32] = Sha256::digest(bytes.into_inner()).into();
        Ok(LeafHash(hash))
    }
}

impl Encode for MerkleTreeLeaf {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.version.encode(&mut writer)?;
        self.leaf.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for MerkleTreeLeaf {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            version: Version::decode(&mut reader)?,
            leaf: Leaf::decode(&mut reader)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Leaf {
    TimestampedEntry(TimestampedEntry),
}

impl Encode for Leaf {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        match self {
            Leaf::TimestampedEntry(entry) => {
                writer.write_all(&[0])?;
                entry.encode(&mut writer)?;
            }
        };
        Ok(())
    }
}

impl Decode for Leaf {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = vec![0u8];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(Leaf::TimestampedEntry(TimestampedEntry::decode(
                &mut reader,
            )?)),
            val => Err(CodecError::UnknownVariant("MerkleLeafType", val as u64)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimestampedEntry {
    timestamp: u64,
    log_entry: LogEntry,
    extensions: CodecVec<u16>,
}

impl Encode for TimestampedEntry {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.timestamp.encode(&mut writer)?;
        self.log_entry.encode(&mut writer)?;
        self.extensions.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for TimestampedEntry {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            timestamp: u64::decode(&mut reader)?,
            log_entry: LogEntry::decode(&mut reader)?,
            extensions: CodecVec::decode(&mut reader)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cert::CertificateChain, tests::get_log_argon2025h2};

    const GOOGLE_GET_ENTRY: &str = include_str!("../../testdata/google-entry.json");
    const CERT_CHAIN_GOOGLE_COM: &str = include_str!("../../testdata/google-chain.pem");

    const ARGON2025H2_STH_0506: &str = "{
        \"tree_size\":1329315675,
        \"timestamp\":1751738269891,
        \"sha256_root_hash\":\"NEFqldTJt2+wE/aaaQuXeADdWVV8IGbwhLublI7QaMY=\",
        \"tree_head_signature\":\"BAMARjBEAiA9rna9/avaKTald7hHrldq8FfB4FDAaNyB44pplv71agIgeD0jj2AhLnvlaWavfFZ3BdUglauz36rFpGLYuLBs/O8=\"
    }";

    #[test]
    fn parse_get_entry_response() {
        let response: GetEntriesResponse = serde_json::from_str(GOOGLE_GET_ENTRY).unwrap();
        assert_eq!(response.entries.len(), 1);

        // Test round trip
        let reencoded = serde_json::to_string(&response).unwrap();
        let response2: GetEntriesResponse = serde_json::from_str(&reencoded).unwrap();
        assert_eq!(response, response2);
    }

    #[test]
    fn generate_precert_comparison() {
        let response: GetEntriesResponse = serde_json::from_str(GOOGLE_GET_ENTRY).unwrap();
        let leaf = response.entries[0].leaf_input.0.0.clone();

        let Leaf::TimestampedEntry(entry) = leaf.leaf;
        let log_entry1 = entry.log_entry;

        let cert2 = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let log_entry2 = cert2.as_precert_entry_v1().unwrap();

        assert_eq!(log_entry1, log_entry2);
    }

    #[test]
    fn audit_sct() {
        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let log = get_log_argon2025h2();
        assert_eq!(log.log_id_v1(), scts[0].log_id());

        log.validate_sct_as_precert_v1(&cert, &scts[0]).unwrap();
    }
}
