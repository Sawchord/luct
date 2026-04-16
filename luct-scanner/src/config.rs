use serde::{Deserialize, Serialize};
use url::Url;
use web_time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub(crate) validate_cert_chain: bool,

    pub(crate) otlsp_url: Option<Url>,
    pub(crate) otlsp_connection_timeout: Duration,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            validate_cert_chain: Default::default(),
            otlsp_url: Some(Url::parse("https://node.luct.dev/otlsp").unwrap()),
            otlsp_connection_timeout: Duration::from_secs(30),
        }
    }
}

impl ScannerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_cert_chain(&self) -> bool {
        self.validate_cert_chain
    }

    pub fn set_validate_cert_chain(mut self) -> Self {
        self.validate_cert_chain = true;
        self
    }

    pub fn otlsp_url(&self) -> &Option<Url> {
        &self.otlsp_url
    }

    pub fn otlsp_connection_timeout(&self) -> &Duration {
        &self.otlsp_connection_timeout
    }
}
