use crate::{ScannerError, log::tiling::TileFetcher, utils::Validated};
use luct_client::{Client, CtClient};
use luct_core::{
    store::OrderedStore,
    v1::{MerkleTreeLeaf, SignedCertificateTimestamp, SignedTreeHead},
};
use std::{fmt, sync::Arc};

pub(crate) mod builder;
pub(crate) mod tiling;

/// Internal structure holding references to per log
/// clients and stores
#[derive(Debug)]
pub(crate) struct ScannerLog<C> {
    log: Arc<ScannerLogInner<C>>,
    tiles: Option<TileFetcher<C>>,
}

pub(crate) struct ScannerLogInner<C> {
    name: String,
    client: CtClient<C>,
    sth_store: Box<dyn OrderedStore<u64, Validated<SignedTreeHead>>>,
}

impl<C> fmt::Debug for ScannerLogInner<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScannerLogInner")
            .field("name", &self.name)
            .finish()
    }
}

impl<C: Client> ScannerLog<C> {
    pub(crate) fn client(&self) -> &CtClient<C> {
        &self.log.client
    }

    #[tracing::instrument(level = "trace")]
    pub(crate) async fn check_sct_inclusion(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &Validated<SignedTreeHead>,
        leaf: &MerkleTreeLeaf,
    ) -> Result<(), ScannerError> {
        match &self.tiles {
            Some(tiles) => Ok(tiles.check_sct_inclusion(sct, sth, leaf).await?),
            None => Ok(self
                .log
                .client
                .check_sct_inclusion_v1(sct, sth, leaf)
                .await?),
        }
    }

    /// Returns the latests STH, if it exists, fetches it otherwise
    #[tracing::instrument(level = "trace")]
    pub(crate) async fn latest_sth(&self) -> Result<Validated<SignedTreeHead>, ScannerError> {
        match self.log.sth_store.last() {
            Some((_, sth)) => Ok(sth),
            None => {
                let sth = self.get_sth().await?;
                self.log.sth_store.insert(sth.tree_size(), sth.clone());
                Ok(sth)
            }
        }
    }

    /// Updates the log to the newest STH
    ///
    /// Checks consistency to the last STH, of one exists
    #[tracing::instrument(level = "trace")]
    pub(crate) async fn update_sth(&self) -> Result<(), ScannerError> {
        let new_sth = self.get_sth().await?;

        if let Some((_, old_sth)) = self.log.sth_store.last() {
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

        self.log.sth_store.insert(new_sth.tree_size(), new_sth);

        Ok(())
    }

    #[tracing::instrument(level = "trace")]
    async fn get_sth(&self) -> Result<Validated<SignedTreeHead>, ScannerError> {
        tracing::debug!("Fetching new STH of log {}", self.log.name);
        match &self.tiles {
            Some(_) => Ok(Validated::new(self.log.client.get_checkpoint().await?)),
            None => Ok(Validated::new(self.log.client.get_sth_v1().await?)),
        }
    }
}
