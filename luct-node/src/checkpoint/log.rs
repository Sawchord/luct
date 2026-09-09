use crate::checkpoint::{
    CheckpointerImpl, config::CheckpointerConfig, error::CheckpointerError,
    metrics::CheckpointerMetrics,
};
use luct_client::{
    ClientError, CtClient,
    tiling::{TileFetchStore, TileFetcher},
};
use luct_core::{
    CtLog,
    store::{OrderedStoreRead, SearchableStoreRead, StoreRead, StoreWrite},
    tiling::Checkpoint,
};
use luct_store::LruCacheStore;
use rand::{Rng, RngExt};
use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone)]
pub(crate) struct CheckpointLog<C: CheckpointerImpl> {
    config: Arc<CheckpointerConfig>,
    metrics: CheckpointerMetrics,
    log: Arc<CheckointLogInner<C>>,
}

struct CheckointLogInner<C: CheckpointerImpl> {
    name: String,
    fetcher: CtClient<C::CheckpointFetcher>,
    store: LruCacheStore<C::CheckpointStore>,
    #[allow(clippy::type_complexity)]
    tiles: Option<TileFetcher<LruCacheStore<TileFetchStore<C::CheckpointFetcher>>>>,
    toc: RwLock<String>,
    last_update: RwLock<SystemTime>,
}

impl<C: CheckpointerImpl + std::fmt::Debug> std::fmt::Debug for CheckpointLog<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckpointLog").finish()
    }
}

// TODO: Schedule new update
impl<C: CheckpointerImpl> CheckpointLog<C> {
    pub async fn new(
        log: &CtLog,
        config: Arc<CheckpointerConfig>,
        metrics: CheckpointerMetrics,
        store: C::CheckpointStore,
        fetcher: C::CheckpointFetcher,
    ) -> Result<Self, CheckpointerError> {
        let name = log.description().to_owned();
        let fetcher = CtClient::new(log.config().clone(), fetcher);

        let tiles = log.config().is_tiling().then(|| {
            TileFetcher::new(LruCacheStore::new(
                TileFetchStore::new(name.clone(), fetcher.clone()),
                config.tile_cache_size,
            ))
        });

        let log = Self {
            config: config.clone(),
            metrics,
            log: Arc::new(CheckointLogInner {
                name,
                fetcher,
                store: LruCacheStore::new(store, config.checkpoint_cache_size),
                tiles,
                last_update: RwLock::new(UNIX_EPOCH),
                toc: RwLock::new("".to_string()),
            }),
        };

        log.update_toc().await;
        log.update_time().await?;

        Ok(log)
    }

    pub(crate) fn serve_toc(&self) -> String {
        self.metrics.toc_served(&self.log.name);
        self.log.toc.read().unwrap().clone()
    }

    pub(crate) async fn serve_checkpoint(&self, tree_size: u64) -> Option<String> {
        let cp = self.log.store.get(tree_size).await;
        self.metrics.checkpoint_served(&self.log.name);

        cp
    }

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

    pub(crate) fn next_update(&self, rng: &mut impl Rng) -> SystemTime {
        // Calculate a random extra duration between minimal and maximal duration
        let duration_range =
            (self.config.maximal_update_interval - self.config.minimal_update_interval).as_secs();
        let extra_duration = Duration::from_secs(rng.random_range(0..duration_range));

        // Calculate the next point in time for an update depending on the last update time
        self.log.last_update.read().unwrap().to_owned()
            + self.config.minimal_update_interval
            + extra_duration
    }

    async fn update_time(&self) -> Result<(), CheckpointerError> {
        let (_tree_size, cp) = match self.log.store.last().await {
            Some(cp) => cp,
            None => {
                *self.log.last_update.write().unwrap() = UNIX_EPOCH;
                return Ok(());
            }
        };

        let cp = Checkpoint::parse(&cp).map_err(ClientError::Checkpoint)?;
        let timestamp = cp
            .timestamp_for_log(self.log.fetcher.log().log_id())
            .map_err(|err| ClientError::SignatureValidationFailed("timestamp", err))?;

        let last_update = UNIX_EPOCH + Duration::from_millis(timestamp);
        self.metrics.set_last_update(&self.log.name, last_update);
        *self.log.last_update.write().unwrap() = last_update;

        Ok(())
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
        self.update_time().await?;

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
