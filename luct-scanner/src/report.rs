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
    pub(crate) signature_validation_time: Option<DateTime<Local>>,
    pub(crate) log_name: Option<String>,
    pub(crate) last_sth: Option<SthReport>,
    pub(crate) inclusion_proof: Option<SthReport>,
    pub(crate) error_description: Option<String>,
}

impl SctReport {
    pub(crate) fn new() -> Self {
        Self {
            cached: false,
            signature_validation_time: None,
            log_name: None,
            last_sth: None,
            inclusion_proof: None,
            error_description: None,
        }
    }

    pub(crate) fn cached(mut self) -> Self {
        self.cached = true;
        self
    }

    pub(crate) fn signature_validation_time(mut self, time: DateTime<Local>) -> Self {
        self.signature_validation_time = Some(time);
        self
    }

    pub(crate) fn log_name(mut self, name: String) -> Self {
        self.log_name = Some(name);
        self
    }

    pub(crate) fn last_sth(mut self, sth: SthReport) -> Self {
        self.last_sth = Some(sth);
        self
    }

    pub(crate) fn inclusion_proof(mut self, sth: SthReport) -> Self {
        self.inclusion_proof = Some(sth);
        self
    }

    pub(crate) fn error_description(mut self, err: String) -> Self {
        self.error_description = Some(err);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SthReport {
    pub(crate) height: u64,
    pub(crate) timestamp: DateTime<Local>,
    pub(crate) verification_time: DateTime<Local>,
}
