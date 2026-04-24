use crate::{
    log::{ScannerLog, ScannerLogInner, tiling::TileFetcher},
    utils::Validated,
};
use luct_client::{Client, CtClient};
use luct_core::{CtLog, CtLogConfig, store::MemoryStore, v1::SignedTreeHead};
use std::sync::Arc;

pub struct LogBuilder<S> {
    name: String,
    config: CtLogConfig,
    sth_store: S,
}

impl LogBuilder<MemoryStore<u64, Validated<SignedTreeHead>>> {
    pub fn new(log: &CtLog) -> Self {
        LogBuilder {
            name: log.description().to_string(),
            config: log.config().clone(),
            sth_store: MemoryStore::default(),
        }
    }
}

impl<S> LogBuilder<S> {
    pub fn with_sth_store<S2>(self, store: S2) -> LogBuilder<S2> {
        LogBuilder::<S2> {
            name: self.name,
            config: self.config,
            sth_store: store,
        }
    }

    pub(crate) fn build<C: Client + Clone>(self, client: &C) -> ScannerLog<C, S> {
        let client = CtClient::new(self.config, client.clone());

        let log = Arc::new(ScannerLogInner {
            name: self.name,
            client,
            sth_store: self.sth_store,
        });

        let tiles = log
            .client
            .log()
            .config()
            .is_tiling()
            .then(|| TileFetcher::new(&log));

        ScannerLog { log, tiles }
    }
}
