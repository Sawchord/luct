use crate::{
    Version,
    store::Hashable,
    utils::{
        codec::{CodecError, Decode, Encode},
        vec::CodecVec,
    },
    v1::LogEntry,
};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Read, Write};

/// See RFC 6962 3.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleTreeLeaf {
    pub(crate) version: Version,
    pub(crate) leaf: Leaf,
}

impl Hashable for MerkleTreeLeaf {
    fn hash(&self) -> [u8; 32] {
        let mut bytes = Cursor::new(vec![]);
        bytes.write_all(&[0]).unwrap();
        self.encode(&mut bytes).unwrap();

        Sha256::digest(bytes.into_inner()).into()
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
    pub(crate) timestamp: u64,
    pub(crate) log_entry: LogEntry,
    pub(crate) extensions: CodecVec<u16>,
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
    use crate::{
        CertificateChain,
        tests::{CERT_CHAIN_GOOGLE_COM, GOOGLE_GET_ENTRY},
        v1::responses::GetEntriesResponse,
    };

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
        let log_entry2 = cert2.as_log_entry_v1(true).unwrap();

        assert_eq!(log_entry1, log_entry2);
    }

    #[test]
    fn test_leaf_creation() {
        let response: GetEntriesResponse = serde_json::from_str(GOOGLE_GET_ENTRY).unwrap();
        let leaf1 = response.entries[0].leaf_input.0.0.clone();

        let cert2 = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let sct2 = cert2.cert().extract_scts_v1().unwrap();
        let leaf2 = cert2.as_leaf_v1(&sct2[0], true).unwrap();

        assert_eq!(leaf1, leaf2)
    }
}
