use std::fmt::{self, Display};

use crate::utils::base64::Base64;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

mod cert;
mod cert_chain;
mod errors;
pub(crate) mod signature;
pub mod store;
pub mod tree;
pub(crate) mod utils;
pub mod v1;
mod version;

pub use cert::{Certificate, CertificateError};
pub use cert_chain::CertificateChain;
pub use errors::{CheckSeverity, Severity};
pub use signature::{HashAlgorithm, SignatureAlgorithm, SignatureValidationError};
pub use version::Version;

// TODO: Introduce a Timestamp type and use it
// TODO: Introduce toplevel types that wrap the inner v1 types to make version agnostic API

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtLog {
    config: CtLogConfig,
    log_id: LogId,
}

impl CtLog {
    pub fn new(config: CtLogConfig) -> Self {
        let log_id = match config.version() {
            Version::V1 => LogId::V1(v1::LogId(Sha256::digest(&config.key.0).into())),
        };

        Self { config, log_id }
    }

    pub fn log_id(&self) -> &LogId {
        &self.log_id
    }

    pub fn config(&self) -> &CtLogConfig {
        &self.config
    }
}

/// Configuration of a [`CtLog`]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtLogConfig {
    #[serde(default)]
    /// The [`Version`] of this log
    version: Version,

    /// The [`Url`] at which the log operates
    url: Url,

    /// Public key used to sign
    key: Base64<Vec<u8>>,

    /// Maximum merge delay
    mmd: u64,

    /// Fetch the values from another url instead
    fetch_url: Option<Url>,
}

impl CtLogConfig {
    /// Create a new [`CtLogConfig`]
    pub fn new(version: Version, url: Url, key: Vec<u8>, mmd: u64) -> Self {
        Self {
            version,
            url,
            key: Base64(key),
            mmd,
            fetch_url: None,
        }
    }

    /// Return the [`Url`] of this log
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Return the fetch [`Url`] for this log
    pub fn fetch_url(&self) -> &Url {
        self.fetch_url.as_ref().unwrap_or(self.url())
    }

    /// Return the [`Version`] of this log
    pub fn version(&self) -> &Version {
        &self.version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogId {
    V1(v1::LogId),
}

impl Display for LogId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogId::V1(log_id) => write!(f, "{log_id}"),
        }
    }
}

impl From<v1::LogId> for LogId {
    fn from(value: v1::LogId) -> Self {
        Self::V1(value)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use base64::{Engine, prelude::BASE64_STANDARD};

    const ARGON2025H1: &str = "
        url = \"https://ct.googleapis.com/logs/us1/argon2025h1/\"
        key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEIIKh+WdoqOTblJji4WiH5AltIDUzODyvFKrXCBjw/Rab0/98J4LUh7dOJEY7+66+yCNSICuqRAX+VPnV8R1Fmg==\"
        mmd = 86400
    ";

    const ARGON2025H2: &str = "
        version = 1
        url = \"https://ct.googleapis.com/logs/us1/argon2025h2/\"
        key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEr+TzlCzfpie1/rJhgxnIITojqKk9VK+8MZoc08HjtsLzD8e5yjsdeWVhIiWCVk6Y6KomKTYeKGBv6xVu93zQug==\"
        mmd = 86400
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

    pub(crate) const CERT_CHAIN_GOOGLE_COM: &str = include_str!("../../testdata/google-chain.pem");
    pub(crate) const CERT_GOOGLE_COM: &str = include_str!("../../testdata/google-cert.pem");
    pub(crate) const PRE_CERT_GOOGLE_COM: &str = include_str!("../../testdata/google-precert.pem");

    pub(crate) const GOOGLE_GET_ENTRY: &str = include_str!("../../testdata/google-entry.json");
    pub(crate) const GOOGLE_STH_CONSISTENCY_PROOF: &str =
        include_str!("../../testdata/sth-consistency-proof.json");
    pub(crate) const GOOGLE_AUDIT_PROOF: &str =
        include_str!("../../testdata/google-precert-audit-proof.json");

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

        let LogId::V1(log_id) = log.log_id();
        assert_eq!(log_id.0.to_vec(), test_log_id)
    }
}
