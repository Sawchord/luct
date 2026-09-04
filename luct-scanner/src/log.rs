use crate::{ScannerConfig, ScannerError, ScannerImpl, log::tiling::TileFetcher, validated::Validated};
use futures::lock::Mutex;
use luct_client::CtClient;
use luct_core::{
    store::{OrderedStoreRead, SearchableStoreRead},
    v1::{MerkleTreeLeaf, SignedCertificateTimestamp, SignedTreeHead},
};
use std::{
    fmt::{self, Debug},
    sync::Arc,
};

pub(crate) mod builder;
pub(crate) mod tiling;
mod update;

/// Internal structure holding references to per log
/// clients and stores
#[derive(Debug)]
pub(crate) struct ScannerLog<S: ScannerImpl> {
    log: Arc<ScannerLogInner<S>>,
    config: Arc<ScannerConfig>,
}

pub(crate) struct ScannerLogInner<S: ScannerImpl> {
    name: String,
    sct_client: CtClient<S::SctClient>,
    sth_client: CtClient<S::SthClient>,
    sth_store: Mutex<S::SthStore>,
    tiles: Option<TileFetcher<S>>,
}

impl<S: ScannerImpl> fmt::Debug for ScannerLogInner<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScannerLogInner")
            .field("name", &self.name)
            .finish()
    }
}

impl<S: ScannerImpl> ScannerLog<S> {
    pub(crate) fn sct_client(&self) -> &CtClient<S::SctClient> {
        &self.log.sct_client
    }

    pub(crate) fn sth_client(&self) -> &CtClient<S::SthClient> {
        &self.log.sth_client
    }

    pub(crate) async fn check_sct_inclusion(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &Validated<SignedTreeHead>,
        leaf: &MerkleTreeLeaf,
    ) -> Result<u64, ScannerError> {
        match &self.log.tiles {
            Some(tiles) => Ok(tiles.check_sct_inclusion(sct, sth, leaf).await?),
            None => Ok(self
                .log
                .sct_client
                .check_sct_inclusion_v1(sct, sth, leaf)
                .await?),
        }
    }

    pub(crate) async fn get_latest_sth(&self) -> Option<Validated<SignedTreeHead>> {
        self.log
            .sth_store
            .lock()
            .await
            .last()
            .await
            .map(|sth| sth.1)
    }

    /// Returns the oldest sth in the store, that still includes the `sct`
    pub(crate) async fn oldest_viable_sth(
        &self,
        sct: &SignedCertificateTimestamp,
    ) -> Option<Validated<SignedTreeHead>> {
        let timestamp = sct.timestamp();

        let tree_head = self
            .log
            .sth_store
            .lock()
            .await
            .find(|_, sth| sth.timestamp() > timestamp)
            .await?;
        Some(tree_head.1)
    }

    async fn fetch_sth(&self) -> Result<Validated<SignedTreeHead>, ScannerError> {
        tracing::debug!("Fetching new STH of log {}", self.log.name);
        match &self.log.tiles {
            Some(_) => Ok(Validated::new(self.log.sth_client.get_checkpoint().await?)),
            None => Ok(Validated::new(self.log.sth_client.get_sth_v1().await?)),
        }
    }
}
