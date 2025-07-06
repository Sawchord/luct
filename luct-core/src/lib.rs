use crate::utils::{
    base64::Base64,
    codec::{CodecError, Decode, Encode},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use url::Url;

pub(crate) mod cert;
pub mod store;
pub mod tree;
pub(crate) mod utils;
pub mod v1;

pub use cert::{Certificate, CertificateChain, CertificateError};

// TODO: Introduce a Timestamp type and use it
// TODO: Introduce a LogId type and use it

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtLog {
    config: CtLogConfig,
    log_id_v1: [u8; 32],
}

impl CtLog {
    pub fn new(config: CtLogConfig) -> Self {
        let log_id = Sha256::digest(&config.key.0).into();
        Self {
            config,
            log_id_v1: log_id,
        }
    }

    pub fn log_id_v1(&self) -> [u8; 32] {
        self.log_id_v1
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtLogConfig {
    url: Url,
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
    use base64::{Engine, prelude::BASE64_STANDARD};

    use super::*;

    const ARGON2025H1: &str = "
        url = \"https://ct.googleapis.com/logs/us1/argon2025h1/\"
        key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEIIKh+WdoqOTblJji4WiH5AltIDUzODyvFKrXCBjw/Rab0/98J4LUh7dOJEY7+66+yCNSICuqRAX+VPnV8R1Fmg==\"
        mdd = 86400
    ";

    const ARGON2025H2: &str = "
        url = \"https://ct.googleapis.com/logs/us1/argon2025h2/\"
        key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEr+TzlCzfpie1/rJhgxnIITojqKk9VK+8MZoc08HjtsLzD8e5yjsdeWVhIiWCVk6Y6KomKTYeKGBv6xVu93zQug==\"
        mdd = 86400
    ";

    pub(crate) fn get_log_argon2025h1() -> CtLog {
        let config = toml::from_str(ARGON2025H1).unwrap();
        CtLog::new(config)
    }

    pub(crate) fn get_log_argon2025h2() -> CtLog {
        let config = toml::from_str(ARGON2025H2).unwrap();
        CtLog::new(config)
    }

    #[test]
    fn ct_log_toml_parse() {
        let log = get_log_argon2025h1();

        let test_log_id = BASE64_STANDARD
            .decode("TnWjJ1yaEMM4W2zU3z9S6x3w4I4bjWnAsfpksWKaOd8=")
            .unwrap();
        assert_eq!(log.log_id_v1().to_vec(), test_log_id)
    }
}
