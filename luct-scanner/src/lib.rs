//! Certificate transparency auditing logic used by luCT firefox extension and CLI tool

#![forbid(unsafe_code)]

use crate::log::{ScannerLog, builder::LogImpls};
use chrono::{DateTime, Local};
use futures::future::try_join_all;
use luct_client::Client;
use luct_core::{CtLog, Fingerprint, LogId, store::SearchableStore, v1::SignedTreeHead};
use std::collections::BTreeMap;
pub use {
    config::{ScannerConfig, ScannerConfigBuilder},
    error::ScannerError,
    report::{Report, SctReport, SthReport},
    utils::Validated,
};

type HashOutput = [u8; 32];

mod config;
mod error;
mod log;
mod report;
mod sth;
mod utils;

/// Bundle trait for [`Scanner`]
///
/// Defines the [`Store`](luct_core::store::Store) and [`Client`] backends to be used by the scanner
pub trait ScannerImpl {
    /// [`Client`] implementation to make connections to logs to
    type Client: Client + Clone;
    /// The [`Store`](luct_core::store::Store) type used to store cached [`Reports`](Report) of audit results
    type ReportStore: SearchableStore<Fingerprint, Report>;
    /// The [`Store`](luct_core::store::Store) use to store [`SignedTreeHeads`](SignedTreeHead)
    type SthStore: SearchableStore<u64, Validated<SignedTreeHead>>;
}

/// The scanner holds the state that is necessary to perform audits as well as the auditing logic
///
/// It is generic over [`ScannerImpl`], which is a bundle trait containing implementations of [`Stores`](luct_core::store::Store)
/// and [`Clients`](Client).
pub struct Scanner<S: ScannerImpl> {
    config: ScannerConfig,
    logs: BTreeMap<LogId, ScannerLog<S>>,
    report_store: S::ReportStore,
    client: S::Client,
    time_source: Box<dyn Fn() -> DateTime<Local>>,
}

#[allow(clippy::type_complexity)]
impl<S: ScannerImpl> Scanner<S> {
    pub fn logs<'a>(&'a self) -> Box<dyn Iterator<Item = &'a CtLog> + 'a> {
        Box::new(self.logs.values().map(|val| val.client().log()))
    }

    pub fn new<F: Fn() -> DateTime<Local> + 'static>(
        config: ScannerConfig,
        report_store: S::ReportStore,
        client: S::Client,
        time_source: F,
    ) -> Self {
        Self {
            config,
            logs: BTreeMap::new(),
            report_store,
            client,
            time_source: Box::new(time_source) as _,
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

        try_join_all(updates).await?;

        Ok(())
    }
}
