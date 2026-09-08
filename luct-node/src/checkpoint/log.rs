use crate::checkpoint::{CheckpointerImpl, error::CheckpointerError};
use luct_client::{
    ClientError, CtClient,
    tiling::{TileFetchStore, TileFetcher},
};
use luct_core::{
    store::{OrderedStoreRead, SearchableStoreRead, StoreWrite},
    tiling::Checkpoint,
};
use luct_store::LruCacheStore;
use std::{
    sync::{Arc, RwLock},
    time::SystemTime,
};

#[derive(Clone)]
pub(crate) struct CheckpointLog<C: CheckpointerImpl> {
    log: Arc<CheckointLogInner<C>>,
    last_update: SystemTime,
}

struct CheckointLogInner<C: CheckpointerImpl> {
    name: String,
    fetcher: CtClient<C::CheckpointFetcher>,
    store: LruCacheStore<C::CheckpointStore>,
    #[allow(clippy::type_complexity)]
    tiles: Option<TileFetcher<LruCacheStore<TileFetchStore<C::CheckpointFetcher>>>>,
    toc: RwLock<String>,
}

impl<C: CheckpointerImpl + std::fmt::Debug> std::fmt::Debug for CheckpointLog<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckpointLog")
            .field("last_update", &self.last_update)
            .finish()
    }
}

// TODO: Load a checkpoint from store and client
//  - Regenerate toc
//  - Get last update
// TODO: Serve toc
// TODO: Schedule new update
impl<C: CheckpointerImpl> CheckpointLog<C> {
    async fn update_toc(&self) -> () {
        let mut toc: Vec<String> = vec![];

        self.log
            .store
            .filter(|tree_size, cp| {
                let cp = match Checkpoint::parse(cp) {
                    Ok(cp) => cp,
                    Err(err) => {
                        tracing::warn!(
                            "Failed to parse a checkpoint at height {}. Error: {:?}",
                            tree_size,
                            err
                        );
                        return false;
                    }
                };

                toc.push(cp.tree_size().to_string());
                false
            })
            .await;

        *self.log.toc.write().unwrap() = toc.join("\n");
    }

    pub(crate) async fn update_sth(&self) -> Result<(), CheckpointerError> {
        let new_cp = self.fetch_sth().await?;

        if let Some(old_cp) = self.log.store.last().await.and_then(|(_, cp)| {
            Checkpoint::parse(&cp)
                .inspect_err(|err| {
                    tracing::warn!("Failed to parse a checkpoint stored on disk: {:?}", err)
                })
                .ok()
        }) && old_cp.tree_size() < new_cp.tree_size()
        {
            tracing::debug!(
                "Updating STH: Checking checkpoint {} against old checkpoint {}",
                new_cp.tree_size(),
                old_cp.tree_size()
            );

            let log = self.log.fetcher.log();
            let old_sth = log
                .cp_to_sth(&old_cp)
                .map_err(|err| ClientError::SignatureValidationFailed("checkpoint", err))?;
            let new_sth = log
                .cp_to_sth(&old_cp)
                .map_err(|err| ClientError::SignatureValidationFailed("checkpoint", err))?;

            match &self.log.tiles {
                Some(tiles) => tiles.check_sth_consistency(&old_sth, &new_sth).await?,
                None => {
                    self.log
                        .fetcher
                        .check_consistency_v1(&old_sth, &new_sth)
                        .await?
                }
            }
        }

        self.log
            .store
            .insert(new_cp.tree_size(), new_cp.to_checkoint_string())
            .await;

        self.update_toc().await;

        Ok(())
    }

    async fn fetch_sth(&self) -> Result<Checkpoint, CheckpointerError> {
        tracing::debug!("Fetching new STH of log {}", self.log.name);
        match &self.log.tiles {
            Some(_) => Ok(self.log.fetcher.get_checkpoint().await?),
            None => {
                let sth = self.log.fetcher.get_sth_v1().await?;
                let cp = self
                    .log
                    .fetcher
                    .log()
                    .sth_to_cp(&sth)
                    .map_err(|err| ClientError::SignatureValidationFailed("checkpoint", err))?;

                Ok(cp)
            }
        }
    }
}
