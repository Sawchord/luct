use std::sync::Arc;

use luct_core::{CertificateChain, v1::SignedCertificateTimestamp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conclusion {
    Safe(String),
    Inconclusive(String),
    Unsafe(String),
    FollowUp(Lead),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lead {
    EmbeddedSct(EmbeddedSct),
}

impl Lead {
    /// Provide a short textual description on the type of lead that is being investigated
    pub fn description(&self) -> String {
        match self {
            Lead::EmbeddedSct(embedded_sct) => format!(
                "SCT of log {} embedded into the certificate",
                embedded_sct.sct.log_id()
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedSct {
    pub(crate) sct: SignedCertificateTimestamp,
    pub(crate) chain: Arc<CertificateChain>,
}
