use crate::{ScannerError, ScannerImpl, log::tiling::TileFetcher, utils::Validated};
use luct_client::CtClient;
use luct_core::{
    store::{OrderedStoreRead, SearchableStoreRead, StoreWrite},
    v1::{MerkleTreeLeaf, SignedCertificateTimestamp, SignedTreeHead},
};
use std::{
    fmt::{self, Debug},
    sync::Arc,
};

pub(crate) mod builder;
pub(crate) mod tiling;

/// Internal structure holding references to per log
/// clients and stores
pub(crate) struct ScannerLog<S: ScannerImpl> {
    log: Arc<ScannerLogInner<S>>,
    tiles: Option<TileFetcher<S>>,
}

impl<S: ScannerImpl> Debug for ScannerLog<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScannerLog")
            .field("log", &self.log)
            .field("tiles", &self.tiles)
            .finish()
    }
}

pub(crate) struct ScannerLogInner<S: ScannerImpl> {
    name: String,
    client: CtClient<S::Client>,
    sth_store: S::SthStore,
}

impl<S: ScannerImpl> fmt::Debug for ScannerLogInner<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScannerLogInner")
            .field("name", &self.name)
            .finish()
    }
}

impl<S: ScannerImpl> ScannerLog<S> {
    pub(crate) fn client(&self) -> &CtClient<S::Client> {
        &self.log.client
    }

    #[tracing::instrument(level = "trace")]
    pub(crate) async fn check_sct_inclusion(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &Validated<SignedTreeHead>,
        leaf: &MerkleTreeLeaf,
    ) -> Result<u64, ScannerError> {
        match &self.tiles {
            Some(tiles) => Ok(tiles.check_sct_inclusion(sct, sth, leaf).await?),
            None => Ok(self
                .log
                .client
                .check_sct_inclusion_v1(sct, sth, leaf)
                .await?),
        }
    }

    pub(crate) fn get_latest_sth(&self) -> Option<Validated<SignedTreeHead>> {
        self.log.sth_store.last().map(|sth| sth.1)
    }

    /// Updates the log to the newest STH
    ///
    /// Checks consistency to the last STH, of one exists
    #[tracing::instrument(level = "trace")]
    pub(crate) async fn update_sth(&self) -> Result<Validated<SignedTreeHead>, ScannerError> {
        let new_sth = self.fetch_sth().await?;

        if let Some((_, old_sth)) = self.log.sth_store.last()
            && old_sth.tree_size() < new_sth.tree_size()
        {
            tracing::debug!(
                "Updating STH: Checking STH {} against old STH {}",
                new_sth.tree_size(),
                old_sth.tree_size()
            );

            match &self.tiles {
                Some(tiles) => tiles.check_sth_consistency(&old_sth, &new_sth).await?,
                None => {
                    self.log
                        .client
                        .check_consistency_v1(&old_sth, &new_sth)
                        .await?
                }
            };
        };

        self.log
            .sth_store
            .insert(new_sth.tree_size(), new_sth.clone());

        Ok(new_sth)
    }

    #[tracing::instrument(level = "trace")]
    pub(crate) fn oldest_viable_sth(
        &self,
        sct: &SignedCertificateTimestamp,
    ) -> Option<Validated<SignedTreeHead>> {
        let timestamp = sct.timestamp();

        let tree_head = self
            .log
            .sth_store
            .find(|_, sth| sth.timestamp() > timestamp)?;
        Some(tree_head.1)
    }

    #[tracing::instrument(level = "trace")]
    async fn fetch_sth(&self) -> Result<Validated<SignedTreeHead>, ScannerError> {
        tracing::debug!("Fetching new STH of log {}", self.log.name);
        match &self.tiles {
            Some(_) => Ok(Validated::new(self.log.client.get_checkpoint().await?)),
            None => Ok(Validated::new(self.log.client.get_sth_v1().await?)),
        }
    }
}
