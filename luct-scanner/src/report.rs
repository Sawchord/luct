use crate::Validated;
use chrono::{DateTime, Local};
use luct_core::v1::SignedTreeHead;
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
    cached: bool,
    signature_validation_time: Option<DateTime<Local>>,
    log_name: Option<String>,
    last_sth: Option<SthReport>,
    inclusion_proof: Option<SthReport>,
    error_description: Option<String>,
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

    // pub(crate) fn cached(mut self) -> Self {
    //     self.cached = true;
    //     self
    // }

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
    height: u64,
    timestamp: DateTime<Local>,
    verification_time: DateTime<Local>,
}

impl From<&Validated<SignedTreeHead>> for SthReport {
    fn from(value: &Validated<SignedTreeHead>) -> Self {
        Self {
            height: value.tree_size(),
            timestamp: DateTime::from_timestamp_millis(value.timestamp() as i64)
                .unwrap()
                .into(),
            verification_time: value.validated_at().into(),
        }
    }
}
