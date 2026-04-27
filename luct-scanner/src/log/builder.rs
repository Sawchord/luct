use crate::{
    ScannerImpl,
    log::{ScannerLog, ScannerLogInner, tiling::TileFetcher},
};
use luct_client::CtClient;
use luct_core::CtLog;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) struct LogImpls<S: ScannerImpl> {
    pub client: S::Client,
    pub sth_store: S::SthStore,
}

impl<S: ScannerImpl> ScannerLog<S> {
    pub fn new(log: &CtLog, impls: LogImpls<S>) -> Self {
        let client = CtClient::new(log.config().clone(), impls.client);

        let log = Arc::new(ScannerLogInner::<S> {
            name: log.description().to_owned(),
            client,
            sth_store: impls.sth_store,
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
