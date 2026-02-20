use crate::{
    log::{ScannerLog, ScannerLogInner, tiling::TileFetcher},
    utils::Validated,
};
use luct_client::{Client, CtClient};
use luct_core::{
    CtLog, CtLogConfig,
    store::{MemoryStore, OrderedStore},
    v1::SignedTreeHead,
};
use std::sync::Arc;

pub struct LogBuilder {
    name: String,
    config: CtLogConfig,
    sth_store: Option<Box<dyn OrderedStore<u64, Validated<SignedTreeHead>>>>,
}

impl LogBuilder {
    pub fn new(log: &CtLog) -> Self {
        Self {
            name: log.description().to_string(),
            config: log.config().clone(),
            sth_store: None,
        }
    }

    pub fn with_sth_store(
        mut self,
        store: impl OrderedStore<u64, Validated<SignedTreeHead>> + 'static,
    ) -> Self {
        self.sth_store = Some(Box::new(store) as _);
        self
    }

    pub(crate) fn build<C: Client + Clone>(self, client: &C) -> ScannerLog<C> {
        let client = CtClient::new(self.config, client.clone());

        let log = Arc::new(ScannerLogInner {
            name: self.name,
            client,
            sth_store: self
                .sth_store
                .unwrap_or_else(|| Box::new(MemoryStore::default())),
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
