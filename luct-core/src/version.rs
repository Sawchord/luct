use std::{
    fmt::Display,
    io::{Read, Write},
};

use serde::{Deserialize, Serialize};

use crate::utils::codec::{CodecError, Decode, Encode};

/// The version of the protocol, that the [`CtLog`] supports
///
/// - `V1` corresponds to RFC 6962
/// - `V2` corresponds to RFC 9162
///
/// Currently, only [`Version::V1`] is supported
///
/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Version {
    #[default]
    V1,
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Version::V1 => serializer.serialize_u8(1),
        }
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let version: u8 = <u8>::deserialize(deserializer)?;
        match version {
            1 => Ok(Version::V1),
            x => Err(serde::de::Error::custom(format!("Unsupported version {x}"))),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V1 => write!(f, "V1"),
        }
    }
}

impl Encode for Version {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let discriminant = match self {
            Version::V1 => 0,
        };
        Ok(writer.write_all(&[discriminant])?)
    }
}

impl Decode for Version {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = vec![0u8];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(Version::V1),
            x => Err(CodecError::UnknownVariant("Version", x as u64)),
        }
    }
}
