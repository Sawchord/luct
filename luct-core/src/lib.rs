use crate::utils::{
    base64::Base64,
    codec::{CodecError, Decode, Encode},
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use url::Url;

pub(crate) mod utils;
pub mod v1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtLog {
    url: Url,
    log_id: Base64<Vec<u8>>,
    key: Base64<Vec<u8>>,
    mdd: u64,
}

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Version {
    V1,
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

#[cfg(test)]
mod tests {
    use super::*;

    const ARGON2025H1: &str = "
url = \"https://ct.googleapis.com/logs/us1/argon2025h1/\"
log_id = \"TnWjJ1yaEMM4W2zU3z9S6x3w4I4bjWnAsfpksWKaOd8=\"
key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEIIKh+WdoqOTblJji4WiH5AltIDUzODyvFKrXCBjw/Rab0/98J4LUh7dOJEY7+66+yCNSICuqRAX+VPnV8R1Fmg==\"
mdd = 86400
    ";

    fn get_log() -> CtLog {
        toml::from_str(ARGON2025H1).unwrap()
    }

    #[test]
    fn ct_log_toml_parse() {
        let _ = get_log();
    }
}
