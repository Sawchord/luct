use crate::{
    log::{ScannerLog, ScannerLogInner, tiling::TileFetcher},
    utils::Validated,
};
use luct_client::{Client, CtClient};
use luct_core::{
    CtLog, CtLogConfig,
    store::{MemoryStore, OrderedStore, Store},
    v1::SignedTreeHead,
};
use std::sync::Arc;

pub struct LogBuilder {
    name: String,
    config: CtLogConfig,
    sth_store: Option<Box<dyn OrderedStore<u64, Validated<SignedTreeHead>>>>,
    root_keys: Option<Box<dyn Store<Vec<u8>, ()>>>,
}

impl LogBuilder {
    pub fn new(log: &CtLog) -> Self {
        Self {
            name: log.description().to_string(),
            config: log.config().clone(),
            sth_store: None,
            root_keys: None,
        }
    }

    pub fn with_sth_store(
        mut self,
        store: impl OrderedStore<u64, Validated<SignedTreeHead>> + 'static,
    ) -> Self {
        self.sth_store = Some(Box::new(store) as _);
        self
    }

    pub fn with_root_key_store(mut self, store: impl Store<Vec<u8>, ()> + 'static) -> Self {
        self.root_keys = Some(Box::new(store) as _);
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
            root_keys: self
                .root_keys
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
