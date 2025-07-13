use crate::utils::{
    base64::Base64,
    codec::{CodecError, Decode, Encode},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt::Display,
    io::{Read, Write},
};
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
    version: Version,
    url: Url,
    key: Base64<Vec<u8>>,
    mdd: u64,
}

impl CtLogConfig {
    /// Return the [`Url`] of this log
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Return the [`Version`] of this log
    pub fn version(&self) -> &Version {
        &self.version
    }
}

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
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

#[cfg(test)]
mod tests {

    use super::*;
    use base64::{Engine, prelude::BASE64_STANDARD};

    const ARGON2025H1: &str = "
        version = 1
        url = \"https://ct.googleapis.com/logs/us1/argon2025h1/\"
        key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEIIKh+WdoqOTblJji4WiH5AltIDUzODyvFKrXCBjw/Rab0/98J4LUh7dOJEY7+66+yCNSICuqRAX+VPnV8R1Fmg==\"
        mdd = 86400
    ";

    const ARGON2025H2: &str = "
        version = 1
        url = \"https://ct.googleapis.com/logs/us1/argon2025h2/\"
        key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEr+TzlCzfpie1/rJhgxnIITojqKk9VK+8MZoc08HjtsLzD8e5yjsdeWVhIiWCVk6Y6KomKTYeKGBv6xVu93zQug==\"
        mdd = 86400
    ";

    pub(crate) const ARGON2025H1_STH2806: &str = "{
    \"tree_size\":1425614114,
    \"timestamp\":1751114416696,
    \"sha256_root_hash\":\"LHtW79pwJohJF5Yn/tyozEroOnho4u3JAGn7WeHSR54=\",
    \"tree_head_signature\":\"BAMARzBFAiEAg4w8LlTFKd3KL6lo5Zde9OupHYNN0DDk8U54PenirI4CIHL8ucpkJw5zFLh8UvLA+Zf+f8Ms+tLsVtzHuqnO0qjm\"
    }";

    pub(crate)const ARGON2025H1_STH2906: &str = "{
    \"tree_size\":1425633154,
    \"timestamp\":1751189445313,
    \"sha256_root_hash\":\"iH90iBSqmtLLTcCwu74RYyJ0rd3oXtLbXlBNqKcJUXA=\",
    \"tree_head_signature\":\"BAMARjBEAiAA/UmelqZIfpd5vBs0CJZGx8kAqUhNppLX/rBVk15DWwIgbyecvj2CUl4YzAEWEoFmUwL9KkrZBZQcQgSNEFDqIgc=\"
    }";

    pub(crate) const CERT_CHAIN_GOOGLE_COM: &str = include_str!("../testdata/google-chain.pem");
    pub(crate) const CERT_GOOGLE_COM: &str = include_str!("../testdata/google-cert.pem");
    pub(crate) const PRE_CERT_GOOGLE_COM: &str = include_str!("../testdata/google-precert.pem");

    pub(crate) const GOOGLE_GET_ENTRY: &str = include_str!("../testdata/google-entry.json");
    pub(crate) const GOOGLE_STH_CONSISTENCY_PROOF: &str =
        include_str!("../testdata/sth-consistency-proof.json");
    pub(crate) const GOOGLE_AUDIT_PROOF: &str =
        include_str!("../testdata/google-precert-audit-proof.json");

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
