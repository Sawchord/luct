use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use web_time::Duration;

/// Configuration values of the [`ScannerConfig`].
///
/// These values determine, how the [`Scanner`](crate::Scanner) behaves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
#[builder(setter(into))]
pub struct ScannerConfig {
    /// Set whether the certificate chain should be validated by the scanner
    ///
    /// This is generally not necessary inside a browser, as the chain has already
    /// been validated by the browser, but e.g. it might make sense when fetching the
    /// chain from a file system.
    #[builder(default)]
    pub(crate) validate_cert_chain: bool,

    /// A STH that is younger than this time is considered fresh an STH that is older mature
    ///
    /// The policy evaluation requires a fresh STH to show that the log is still active
    /// If the STH against which the inclusion proof has been made is mature, it will not require
    /// additional STH validations
    #[builder(default = "Duration::from_secs(60 * 60 * 24)")]
    pub(crate) sth_freshness_threshold: Duration,

    /// If the logs newest STH is older than this time, it will attempt to fetch a fresher value
    ///
    /// This value must not be larger than `sth_freshness_theshold`
    #[builder(default = "Duration::from_secs(60 * 60 * 8)")]
    pub(crate) sth_update_threshold: Duration,
}

impl ScannerConfig {
    /// Return a [`ScannerConfigBuilder`]
    pub fn builder() -> ScannerConfigBuilder {
        ScannerConfigBuilder::default()
    }

    /// Returns `true`, if certificate chain validation is activated
    pub fn validate_cert_chain(&self) -> bool {
        self.validate_cert_chain
    }
}
