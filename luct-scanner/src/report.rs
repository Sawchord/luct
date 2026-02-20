use crate::Validated;
use chrono::{DateTime, Local, TimeDelta};
use luct_core::v1::SignedTreeHead;
use luct_store::StringStoreValue;
use serde::{Deserialize, Serialize};
use web_time::UNIX_EPOCH;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub(crate) ca_name: String,
    pub(crate) not_before: DateTime<Local>,
    pub(crate) not_after: DateTime<Local>,
    pub(crate) scts: Vec<SctReport>,
}

impl Report {
    pub fn evaluate_policy(&self, time: DateTime<Local>) -> Result<(), String> {
        let num_expected_scts = match self.not_after - self.not_before {
            time if time <= TimeDelta::days(180) => 2,
            _ => 3,
        };

        let num_scts = self
            .scts
            .iter()
            .filter(|sct| sct.signature_validation_time.is_some())
            .count();

        if num_scts < num_expected_scts {
            return Err(format!(
                "Insufficient number of SCTs from known logs. Expected {} but got {}",
                num_expected_scts, num_scts
            ));
        }

        // TODO: Check that expiration date matches logs bracket?

        let (old_inclusion_proofs, fresh_inclusion_proofs) = self
            .scts
            .iter()
            // Filter out sct reports that correspond to logs that don't have a recent sth
            .filter(|sct_report| {
                sct_report.latest_sth.as_ref().is_some_and(|sth_report| {
                    sth_report.verification_time > time - TimeDelta::hours(24)
                })
            })
            .filter_map(|sct_report| sct_report.inclusion_proof.as_ref())
            .partition::<Vec<_>, _>(|sth_report| {
                sth_report.verification_time < time - TimeDelta::hours(24)
            });

        if old_inclusion_proofs.is_empty() && fresh_inclusion_proofs.len() < 2 {
            return Err(
                "Insufficient number of inclusion proofs with fresh sths could be verified!"
                    .to_string(),
            );
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SctReport {
    cached: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    signature_validation_time: Option<DateTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    log_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    latest_sth: Option<SthReport>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    inclusion_proof: Option<SthReport>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    error_description: Option<String>,
}

impl StringStoreValue for SctReport {
    fn serialize_value(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn deserialize_value(value: &str) -> Option<Self> {
        serde_json::from_str(value).ok()
    }
}

impl SctReport {
    pub(crate) fn new() -> Self {
        Self {
            cached: false,
            signature_validation_time: None,
            log_name: None,
            latest_sth: None,
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

    pub(crate) fn latest_sth(mut self, sth: SthReport) -> Self {
        self.latest_sth = Some(sth);
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
            verification_time: DateTime::from_timestamp_millis(
                value
                    .validated_at()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
            )
            .unwrap()
            .into(),
        }
    }
}
