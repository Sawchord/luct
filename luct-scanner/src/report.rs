use crate::{Scanner, ScannerImpl, Validated, utils::system_time_to_date_time};
use chrono::{DateTime, Utc};
use luct_core::{
    CertificateChain, LogId,
    store::{SearchableStoreRead, StoreWrite},
    v1::SignedTreeHead,
};
use luct_store::StringStoreValue;
use serde::{Deserialize, Serialize};
use web_time::SystemTime;

mod evaluate;
mod generate;

impl<S: ScannerImpl> Scanner<S> {
    pub async fn refresh_reports(&self) {
        let now = system_time_to_date_time(SystemTime::now());

        let outdated_reports = self
            .report_store
            .filter(|_, value| value.not_after < now)
            .await;

        if !outdated_reports.is_empty() {
            tracing::info!(
                "Found {} outdated reports. Will remove them.",
                outdated_reports.len()
            )
        }

        for (fp, report) in outdated_reports {
            self.report_store.delete(fp).await;
            tracing::info!("Removed outdated report for {}", report.cert_subject)
        }

        tracing::info!("Refreshed reports")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub(crate) ca_issuer: String,
    pub(crate) ca_subject: String,
    pub(crate) cert_issuer: String,
    pub(crate) cert_subject: String,
    pub(crate) fingerprint: String,
    pub(crate) ca_fingerprint: String,
    pub(crate) not_before: DateTime<Utc>,
    pub(crate) not_after: DateTime<Utc>,
    // TODO: Precert fingerprint
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub(crate) scts: Vec<SctReport>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) error_description: Option<String>,
}

impl Report {
    pub fn get_error(&self) -> Option<String> {
        self.error_description.clone()
    }

    pub(crate) fn error_description(mut self, err: String) -> Self {
        self.error_description = Some(err);
        self
    }
}

impl From<&CertificateChain> for Report {
    fn from(chain: &CertificateChain) -> Self {
        let (not_before, not_after) = chain.cert().get_validity();

        Self {
            ca_issuer: chain.root().get_issuer_name(),
            ca_subject: chain.root().get_subject_name(),
            cert_issuer: chain.cert().get_issuer_name(),
            cert_subject: chain.cert().get_subject_name(),
            fingerprint: chain.cert().fingerprint_sha256().to_string(),
            ca_fingerprint: chain.root().fingerprint_sha256().to_string(),
            not_before,
            not_after,
            scts: vec![],
            error_description: None,
        }
    }
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
    pub(crate) log_id: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) signature_validation_time: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) log_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) latest_sth: Option<SthReport>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) inclusion_proof: Option<SthReport>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub(crate) error_description: Option<String>,
}

impl SctReport {
    pub(crate) fn new(log_id: LogId) -> Self {
        Self {
            log_id: log_id.to_string(),
            signature_validation_time: None,
            log_name: None,
            latest_sth: None,
            index: None,
            inclusion_proof: None,
            error_description: None,
        }
    }

    pub(crate) fn signature_validation_time(mut self, time: DateTime<Utc>) -> Self {
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

    pub(crate) fn index(mut self, index: u64) -> Self {
        self.index = Some(index);
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

    pub(crate) fn set_error_description(&mut self, err: String) {
        self.error_description = Some(err);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SthReport {
    height: u64,
    timestamp: DateTime<Utc>,
    verification_time: DateTime<Utc>,
}

impl From<&Validated<SignedTreeHead>> for SthReport {
    fn from(value: &Validated<SignedTreeHead>) -> Self {
        Self {
            height: value.tree_size(),
            timestamp: DateTime::from_timestamp_millis(value.timestamp() as i64).unwrap(),
            verification_time: system_time_to_date_time(value.validated_at()),
        }
    }
}

// TODO: Tests for policy evaluation
