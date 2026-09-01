use crate::{
    ScannerImpl,
    log::{ScannerLog, ScannerLogInner, tiling::TileFetcher},
};
use futures::lock::Mutex;
use luct_client::CtClient;
use luct_core::CtLog;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct LogImpls<S: ScannerImpl> {
    pub client: S::SctClient,
    pub sth_store: S::SthStore,
}

impl<S: ScannerImpl> ScannerLog<S> {
    pub fn new(log: &CtLog, impls: LogImpls<S>) -> Self {
        let config = log.config();
        let name = log.description().to_owned();

        let client = CtClient::new(log.config().clone(), impls.client);
        let tiles = config
            .is_tiling()
            .then(|| TileFetcher::new2(name.clone(), client.clone()));

        let log = Arc::new(ScannerLogInner::<S> {
            name,
            client,
            sth_store: Mutex::new(impls.sth_store),
            tiles,
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
