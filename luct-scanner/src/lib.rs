use crate::log::{ScannerLog, builder::LogImpls};
use chrono::DateTime;
use futures::future::{self, join_all};
use luct_client::{Client, ClientError};
use luct_core::{
    CertificateChain, CertificateError, CtLog, CtLogConfig, Fingerprint, LogId,
    store::{SearchableStore, StoreRead, StoreWrite},
    tiling::TilingError,
    v1::{SignedCertificateTimestamp, SignedTreeHead},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug, sync::Arc};
use thiserror::Error;
use web_time::{Duration, SystemTime, UNIX_EPOCH};
pub use {
    config::{ScannerConfig, ScannerConfigBuilder},
    report::{Report, SctReport, SthReport},
    utils::Validated,
};

type HashOutput = [u8; 32];

mod config;
mod log;
mod report;
mod utils;

pub trait ScannerImpl {
    type Client: Client + Clone;
    type ReportStore: SearchableStore<Fingerprint, Report>;
    type SthStore: SearchableStore<u64, Validated<SignedTreeHead>>;
}

pub struct Scanner<S: ScannerImpl> {
    config: ScannerConfig,
    logs: BTreeMap<LogId, ScannerLog<S>>,
    report_store: S::ReportStore,
    client: S::Client,
}

#[allow(clippy::type_complexity)]
impl<S: ScannerImpl> Scanner<S> {
    pub fn logs<'a>(&'a self) -> Box<dyn Iterator<Item = &'a CtLog> + 'a> {
        Box::new(self.logs.values().map(|val| val.client().log()))
    }

    pub fn new_with_client(
        config: ScannerConfig,
        report_store: S::ReportStore,
        client: S::Client,
    ) -> Self {
        Self {
            config,
            logs: BTreeMap::new(),
            report_store,
            client,
        }
    }

    pub fn add_log(&mut self, log: &CtLog, sth_store: S::SthStore) -> &mut Self {
        let impls = LogImpls {
            client: self.client.clone(),
            sth_store,
        };
        let scanner_log = ScannerLog::new(log, impls);
        let log_id = scanner_log.client().log().log_id().clone();

        self.logs.insert(log_id, scanner_log);
        self
    }

    /// Updates all log's STHs
    pub async fn refresh_all_logs(&self) -> Result<(), ScannerError> {
        let updates = self
            .logs
            .values()
            .map(|log| log.update_sth())
            .collect::<Vec<_>>();

        future::try_join_all(updates).await?;

        Ok(())
    }

    pub async fn collect_report_pem(&self, data: &str) -> Result<Report, ScannerError> {
        let cert_chain = Arc::new(CertificateChain::from_pem_chain(data)?);

        if self.config.validate_cert_chain {
            cert_chain.verify_chain()?;
        }

        self.collect_report(cert_chain).await
    }

    pub async fn collect_report(
        &self,
        chain: Arc<CertificateChain>,
    ) -> Result<Report, ScannerError> {
        let cert = chain.cert();
        let cert_fp = cert.fingerprint_sha256();

        if let Some(report) = self.report_store.get(&cert_fp) {
            tracing::debug!("Found report for {} in cache", cert_fp.to_string());
            return Ok(report);
        }

        let (not_before, not_after) = cert.get_validity();
        let embedded_scts = cert.extract_scts_v1()?;

        let scts = join_all(
            embedded_scts
                .into_iter()
                .map(|sct| self.collect_embedded_sct_report(sct, &chain)),
        )
        .await;

        let report = Report {
            ca_issuer: chain.root().get_issuer_name(),
            ca_subject: chain.root().get_subject_name(),
            cert_issuer: chain.cert().get_issuer_name(),
            cert_subject: chain.cert().get_subject_name(),
            fingerprint: chain.cert().fingerprint_sha256().to_string(),
            not_before: not_before.into(),
            not_after: not_after.into(),
            scts,
            error_description: None,
        };

        self.report_store.insert(cert_fp, report.clone());
        Ok(report)
    }

    pub(crate) async fn collect_embedded_sct_report(
        &self,
        sct: SignedCertificateTimestamp,
        chain: &Arc<CertificateChain>,
    ) -> SctReport {
        let report = SctReport::new(sct.log_id());

        // Find the log this sct belongs to
        let Some(log) = self.logs.get(&sct.log_id()) else {
            return report.error_description(format!("No log with id {} known", sct.log_id()));
        };
        let log_name = log.client().log().description().to_string();
        let report = report.log_name(log_name);

        // Validate the signature
        if let Err(err) = log.client().log().validate_sct_v1(chain, &sct, true) {
            return report.error_description(err.to_string());
        };
        let report = report.signature_validation_time(
            DateTime::from_timestamp_millis(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
            )
            .unwrap()
            .into(),
        );

        // Get a fresh sth
        let fresh_sth = match self.get_fresh_sth(log).await {
            Ok(sth) => sth,
            Err(err) => return report.error_description(err.to_string()),
        };
        let report = report.latest_sth(SthReport::from(&fresh_sth));

        let leaf = match chain.as_leaf_v1(&sct, true) {
            Err(err) => {
                return report.error_description(err.to_string());
            }
            Ok(leaf) => leaf,
        };

        // Check inclusion
        let oldest_sth = log.oldest_viable_sth(&sct).unwrap_or(fresh_sth);
        let report = match log.check_sct_inclusion(&sct, &oldest_sth, &leaf).await {
            Ok(index) => report.index(index),
            Err(err) => return report.error_description(err.to_string()),
        };

        report.inclusion_proof(SthReport::from(&oldest_sth))
    }

    /// Get a fresh STH
    ///
    /// Checks whether the latest STH is still new enough.
    /// If it is too old, it will fetch a fresh one
    async fn get_fresh_sth(
        &self,
        log: &ScannerLog<S>,
    ) -> Result<Validated<SignedTreeHead>, ScannerError> {
        let log_name = log.client().log().description();

        // If we have no STH whatsoever, simply fetch it
        let Some(last_sth) = log.get_latest_sth() else {
            tracing::debug!("No prior known STHs for {}, fetching fresh STH", log_name);
            return log.update_sth().await;
        };

        let now = SystemTime::now();
        let timestamp = UNIX_EPOCH + Duration::from_millis(last_sth.timestamp());
        if timestamp + self.config.sth_update_threshold < now {
            tracing::debug!("Updating old STH for log {}", log_name);
            log.update_sth().await
        } else {
            Ok(last_sth)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerBuilder {
    config: ScannerConfig,
    logs: Vec<CtLogConfig>,
}

#[derive(Debug, Clone, Error)]
pub enum ScannerError {
    #[error("Invalid certificate: {0}")]
    CertificateError(#[from] CertificateError),

    #[error("HTTP client error {0}")]
    ClientError(#[from] ClientError),

    #[error("Failed to construct proof from tiles {0}")]
    TilingError(#[from] TilingError),
}
