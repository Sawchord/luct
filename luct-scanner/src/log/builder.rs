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
    pub sct_client: S::SctClient,
    pub sth_client: S::SthClient,
    pub sth_store: S::SthStore,
}

impl<S: ScannerImpl> ScannerLog<S> {
    pub fn new(log: &CtLog, impls: LogImpls<S>) -> Self {
        let config = log.config();
        let name = log.description().to_owned();

        let sct_client = CtClient::new(log.config().clone(), impls.sct_client);
        let sth_client = CtClient::new(log.config().clone(), impls.sth_client);
        let tiles = config
            .is_tiling()
            .then(|| TileFetcher::new(name.clone(), sct_client.clone(), sth_client.clone()));

        let log = Arc::new(ScannerLogInner::<S> {
            name,
            sct_client,
            sth_client,
            sth_store: Mutex::new(impls.sth_store),
            tiles,
        });

        ScannerLog { log }
    }
}
