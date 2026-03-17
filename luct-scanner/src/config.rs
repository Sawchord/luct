use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScannerConfig {
    pub(crate) validate_cert_chain: bool,
}

impl ScannerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_cert_chain(mut self) -> Self {
        self.validate_cert_chain = true;
        self
    }
}
