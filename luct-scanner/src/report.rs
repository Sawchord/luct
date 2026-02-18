use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub(crate) ca_name: String,
    pub(crate) not_before: DateTime<Local>,
    pub(crate) not_after: DateTime<Local>,
    pub(crate) scts: Vec<SctReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SctReport {
    pub(crate) cached: bool,
    pub(crate) signature_validates: bool,
    pub(crate) signature_validation_time: DateTime<Local>,
    pub(crate) log_name: String,
    pub(crate) last_sth: SthReport,
    pub(crate) inclusion_proof: Option<SthReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SthReport {
    pub(crate) height: u64,
    pub(crate) timestamp: DateTime<Local>,
    pub(crate) verification_time: DateTime<Local>,
}
