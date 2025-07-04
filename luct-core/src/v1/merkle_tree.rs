use crate::{
    Version,
    utils::{
        base64::Base64,
        codec::{Codec, CodecError, Decode, Encode},
        vec::CodecVec,
    },
    v1::LogEntry,
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

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
pub struct MerkleTreeLeaf {
    version: Version,
    leaf: Leaf,
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

    const GOOGLE_GET_ENTRY: &str = include_str!("../../testdata/google-entry.json");

    #[test]
    fn parse_get_entry_response() {
        let response: GetEntriesResponse = serde_json::from_str(GOOGLE_GET_ENTRY).unwrap();
        assert_eq!(response.entries.len(), 1);

        // Test round trip
        let reencoded = serde_json::to_string(&response).unwrap();
        let response2: GetEntriesResponse = serde_json::from_str(&reencoded).unwrap();
        assert_eq!(response, response2);
    }

    // const ARGON2025H1_CONSISTENCY: &str = "{
    // \"consistency\":[
    //     \"/qxhAu1l2bHdO41AWkZ1+D2xn8eqDXFsEZU99tz0Zwg=\",
    //     \"96OsxcsJgncKCPuBr9b4it0tXeZM/yEiiKUx84xgmqI=\",
    //     \"KPO2TCYRlSLiKhw3FKG/QGM3/XOcqV0Yo5cX/i6Te2s=\",
    //     \"JYxzHyaYvCJulAD30dtHlG882yOBxMhnsFEOqkxx8n8=\",
    //     \"MTJ/W3MuAX7J6FCKOWzP7qSq/mXmqI+qPKN4b8SgBIY=\",
    //     \"aW9uOA5He4q7gbrTugpuZbwXhqJ9W9mpw/RRB6REcwU=\",
    //     \"pAbSFTjehDkKMjqlbqe/Ywvf4FirNcxKJGQbKh0CbPc=\",
    //     \"9ZWAvFdlYx0PvcgR83frVhiQ51VoICKcR1uRrv5AHaA=\",
    //     \"wo+auyrwkSf6uVuIzs5MsNHlCGQNlufvVDvdo4xg/mQ=\",
    //     \"/COh2xbeLPPY5IlqyQHcFeqU2j9cxQl/F1g20wb4Mn8=\",
    //     \"6scEK427tO6n3vzUvBrQmK18nGrBpt48HvXgHjpqyEI=\",
    //     \"m/kWyQAkeEt9W76mRtAFB6jNqgEhIa8Xq9h9E3pEp30=\",
    //     \"SVjnMqYjTAeiC+1K7a2k4qNHlDaupGUnF0F7G7uC9B8=\",
    //     \"JkiceIKDdHgsV9ig+x9X8Fj4q1r2MoXZvYxcgERyuEo=\",
    //     \"s3aSjL7PvFiMGhstflI/w6vxDLv/PjlrJlIa5rRpem4=\",
    //     \"HDTpANRrM9TjrRbTNbPvxTvwPacBYHtoV7eV4Fa9hKc=\",
    //     \"CxFTU+6XplS4HH5NrENZ6cPnd8rUBs4Kt1jVpBBY+Ck=\",
    //     \"vOEAxv7qaKOV5Jaxg/6VMQC2LWnaLxZtsjPpypyyTHM=\",
    //     \"AeW0LJfjoHjbeiUPVsM7QncUrPY46MLNPcy/uycRATo=\",
    //     \"+ezaOjeUzMr7biFsbJxlFVDD7KGkgS+huicyz3y3BVs=\",
    //     \"99jnACdSzOOoKrRf6DTSiR58OuO/HD3Me7uaXTjtScQ=\",
    //     \"mMadhMt51T9DaCw9gliGKkXQQ+zTZCUKKQuYaOt893I=\",
    //     \"PJBkY/VH7A5ZhIfqCtUcuc/xxoK9dgnsIrpoF16pzNU=\",
    //     \"FO1qamBJREqIDiaC0nJUvhtYgwhTbv4mKlfKWSzerFs=\"
    // ]}";
}
