//! Certificate transparency auditing logic used by luCT firefox extension and CLI tool

#![forbid(unsafe_code)]

use crate::log::{ScannerLog, builder::LogImpls};
use chrono::{DateTime, Utc};
use futures::future::try_join_all;
use luct_client::Client;
use luct_core::{CtLog, Fingerprint, LogId, store::SearchableStore, v1::SignedTreeHead};
use std::{collections::BTreeMap, sync::Arc};
use web_time::SystemTime;
pub use {
    config::{ScannerConfig, ScannerConfigBuilder},
    error::ScannerError,
    report::{Report, SctReport, SthReport},
    validated::Validated,
};

mod config;
mod error;
mod log;
mod report;
mod stats;
mod utils;
mod validated;

/// Bundle trait for [`Scanner`]
///
/// Defines the [`Store`](luct_core::store::Store) and [`Client`] backends to be used by the scanner
pub trait ScannerImpl {
    /// The [`Client`] that makes the connections when fetching Scts.
    type SctClient: Client + Clone;

    /// The [`Client`] that fetches STHs, when there is no connection to an SCT.
    type SthClient: Client + Clone;

    /// The [`Store`](luct_core::store::Store) type used to store cached [`Reports`](Report) of audit results
    type ReportStore: SearchableStore<Key = Fingerprint, Value = Report>;

    /// The [`Store`](luct_core::store::Store) type used to store cached [`Reports`](Report) that should
    /// persist beyond the current session (i.e. in private browsing)
    type NonpersistentReportStore: SearchableStore<Key = Fingerprint, Value = Report>;

    /// The [`Store`](luct_core::store::Store) use to store [`SignedTreeHeads`](SignedTreeHead)
    type SthStore: SearchableStore<Key = u64, Value = Validated<SignedTreeHead>>;
}

/// The scanner holds the state that is necessary to perform audits as well as the auditing logic
///
/// It is generic over [`ScannerImpl`], which is a bundle trait containing implementations of [`Stores`](luct_core::store::Store)
/// and [`Clients`](Client).
pub struct Scanner<S: ScannerImpl> {
    config: Arc<ScannerConfig>,
    logs: BTreeMap<LogId, ScannerLog<S>>,
    report_store: S::ReportStore,
    priv_report_store: S::NonpersistentReportStore,
    sct_client: S::SctClient,
    sth_client: S::SthClient,
    time_source: Box<dyn Fn() -> DateTime<Utc>>,
}

#[allow(clippy::type_complexity)]
impl<S: ScannerImpl> Scanner<S> {
    pub fn logs<'a>(&'a self) -> Box<dyn Iterator<Item = &'a CtLog> + 'a> {
        Box::new(self.logs.values().map(|val| val.sct_client().log()))
    }

    pub fn new<F: Fn() -> DateTime<Utc> + 'static>(
        config: ScannerConfig,
        report_store: S::ReportStore,
        priv_report_store: S::NonpersistentReportStore,
        sct_client: S::SctClient,
        sth_client: S::SthClient,
        time_source: F,
    ) -> Self {
        Self {
            config: Arc::new(config),
            logs: BTreeMap::new(),
            report_store,
            priv_report_store,
            sct_client,
            sth_client,
            time_source: Box::new(time_source) as _,
        }
    }

    pub fn add_log(&mut self, log: &CtLog, sth_store: S::SthStore) -> &mut Self {
        let impls = LogImpls {
            config: self.config.clone(),
            sct_client: self.sct_client.clone(),
            sth_client: self.sth_client.clone(),
            sth_store,
        };
        let scanner_log = ScannerLog::new(log, impls);
        let log_id = scanner_log.sct_client().log().log_id().clone();

        self.logs.insert(log_id, scanner_log);
        self
    }

    /// Updates all log's to the latest STHs
    pub async fn update_all_sths(&self) -> Result<(), ScannerError> {
        let updates = self
            .logs
            .values()
            .map(|log| log.update_sth())
            .collect::<Vec<_>>();

        try_join_all(updates).await?;

        Ok(())
    }

    pub async fn refesh_all_sths(&self) -> Result<(), ScannerError> {
        let now = SystemTime::now();
        let updates = self
            .logs
            .values()
            .map(|log| log.get_fresh_sth(now, None))
            .collect::<Vec<_>>();

        try_join_all(updates).await?;

        Ok(())
    }
}
