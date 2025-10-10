use luct_core::{CertificateChain, CheckSeverity, Severity, v1::SignedCertificateTimestamp};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    sync::Arc,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerConfig {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeadResult {
    Conclusion(Conclusion),
    FollowUp(Vec<Lead>),
}

impl From<Conclusion> for LeadResult {
    fn from(value: Conclusion) -> Self {
        Self::Conclusion(value)
    }
}

impl<E: CheckSeverity> From<E> for Conclusion {
    fn from(value: E) -> Self {
        match value.severity() {
            Severity::Inconclusive => Conclusion::Inconclusive(format!("{value}")),
            Severity::Unsafe => Conclusion::Unsafe(format!("{value}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conclusion {
    Safe(String),
    Inconclusive(String),
    Unsafe(String),
}

impl Conclusion {
    pub fn is_safe(&self) -> bool {
        matches!(self, Conclusion::Safe(_))
    }
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

impl PartialOrd for Conclusion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Conclusion {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Conclusion::Safe(_), Conclusion::Safe(_)) => Ordering::Equal,
            (Conclusion::Safe(_), _) => Ordering::Greater,
            (Conclusion::Inconclusive(_), Conclusion::Safe(_)) => Ordering::Less,
            (Conclusion::Inconclusive(_), Conclusion::Inconclusive(_)) => Ordering::Equal,
            (Conclusion::Inconclusive(_), Conclusion::Unsafe(_)) => Ordering::Greater,
            (Conclusion::Unsafe(_), Conclusion::Unsafe(_)) => Ordering::Equal,
            (Conclusion::Unsafe(_), _) => Ordering::Less,
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
