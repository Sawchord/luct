use crate::{Scanner, ScannerImpl, Validated};
use chrono::{DateTime, Local, TimeDelta};
use luct_core::{LogId, v1::SignedTreeHead};
use luct_store::StringStoreValue;
use serde::{Deserialize, Serialize};
use web_time::{Duration, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub(crate) ca_issuer: String,
    pub(crate) ca_subject: String,
    pub(crate) cert_issuer: String,
    pub(crate) cert_subject: String,
    pub(crate) fingerprint: String,
    pub(crate) not_before: DateTime<Local>,
    pub(crate) not_after: DateTime<Local>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) scts: Vec<SctReport>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) error_description: Option<String>,
}

impl<S: ScannerImpl> Scanner<S> {
    // TODO: Evaluate the policy right when creating the report and put the errors into the
    // error fields of the report
    pub fn evaluate_policy(
        &self,
        report: Report,
        current_time: DateTime<Local>,
    ) -> Result<(), String> {
        // TODO: Check that expiration date matches logs submission bracket?

        // Calculate the number of scts we expect
        let num_expected_scts = match report.not_after - report.not_before {
            time if time <= TimeDelta::days(180) => 2,
            _ => 3,
        };

        // Calculate the number of scts that the report contains from known logs
        // TODO: Make sure that the logs are from different operators
        let num_scts_from_known_logs = report
            .scts
            .iter()
            // NOTE: Having a signature that passed validation means the log is known
            .filter(|sct| sct.signature_validation_time.is_some())
            .count();

        // Check that we have enough SCTs from known logs
        if num_scts_from_known_logs < num_expected_scts {
            return Err(format!(
                "Insufficient number of SCTs from known logs. Expected {} but got {}",
                num_expected_scts, num_scts_from_known_logs
            ));
        }

        let mut fresh_inclusion_proofs = 0;
        let mut old_inclusion_proofs = 0;
        for sct in report.scts.iter() {
            // Scts with error cannot be valid
            if sct.error_description.is_some() {
                continue;
            }

            // Check that the SCT has a a fresh STH
            let Some(latest_sth) = &sct.latest_sth else {
                // Could not find a fresh STH for this SCT
                continue;
            };
            if latest_sth.verification_time
                < current_time - time_delta_from_duration(self.config.sth_freshness_threshold)
            {
                // The logs latest STH is too old and the log is considered state
                continue;
            }

            // Check whether the proofs are old or fresh
            let Some(inclusion_proof) = &sct.inclusion_proof else {
                // Could not find an inclusion proof for this SCT
                continue;
            };
            if inclusion_proof.verification_time
                < current_time - time_delta_from_duration(self.config.sth_freshness_threshold)
            {
                old_inclusion_proofs += 1;
            } else {
                fresh_inclusion_proofs += 1;
            }
        }

        if old_inclusion_proofs == 0 && fresh_inclusion_proofs < num_expected_scts {
            return Err(
                "Insufficient number of inclusion proofs with fresh sths could be verified!"
                    .to_string(),
            );
        }

        Ok(())
    }
}

fn time_delta_from_duration(duration: Duration) -> TimeDelta {
    TimeDelta::new(duration.as_secs() as i64, duration.subsec_nanos())
        .expect("Failed to translate duration into timedelta")
}

impl StringStoreValue for Report {
    fn serialize_value(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn deserialize_value(value: &str) -> Option<Self> {
        serde_json::from_str(value).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SctReport {
    log_id: String,
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

impl SctReport {
    pub(crate) fn new(log_id: LogId) -> Self {
        Self {
            log_id: log_id.to_string(),
            signature_validation_time: None,
            log_name: None,
            latest_sth: None,
            inclusion_proof: None,
            error_description: None,
        }
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

// TODO: Tests for policy evaluation
