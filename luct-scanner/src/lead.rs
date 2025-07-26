use luct_core::{CertificateChain, v1::SignedCertificateTimestamp};
use std::{
    fmt::{self, Display},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeadResult {
    Conclusion(Conclusion),
    FollowUp(Vec<Lead>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conclusion {
    Safe(String),
    Inconclusive(String),
    Unsafe(String),
}

impl Display for Conclusion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Conclusion::Safe(reason) => write!(f, "Safe: {reason}"),
            Conclusion::Inconclusive(reason) => write!(f, "Inconclusive: {reason}"),
            Conclusion::Unsafe(reason) => write!(f, "UNSAFE!: {reason}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lead {
    EmbeddedSct(EmbeddedSct),
}

impl Display for Lead {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Lead {
    /// Provide a short textual description on the type of lead that is being investigated
    pub fn description(&self) -> String {
        match self {
            Lead::EmbeddedSct(embedded_sct) => {
                format!("Embedded SCT for log \"{}\"", embedded_sct.sct.log_id())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedSct {
    pub(crate) sct: SignedCertificateTimestamp,
    pub(crate) chain: Arc<CertificateChain>,
}
