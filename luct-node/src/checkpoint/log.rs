use crate::checkpoint::CheckpointerImpl;
use luct_client::tiling::{TileFetchStore, TileFetcher};
use luct_store::LruCacheStore;
use std::{sync::Arc, time::SystemTime};

#[derive(Clone)]
pub(crate) struct CheckpointLog<C: CheckpointerImpl> {
    inner: Arc<CheckointLogInner<C>>,
    last_update: SystemTime,
    toc: String,
}

impl<C: CheckpointerImpl + std::fmt::Debug> std::fmt::Debug for CheckpointLog<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CheckpointLog")
            .field("last_update", &self.last_update)
            .field("toc", &self.toc)
            .finish()
    }
}

// TODO: Load a checkpoint from store and client
//  - Regenerate toc
//  - Get last update
// TODO: Generate and serve toc
// TODO: Make an update
// TODO: Schedule new update

struct CheckointLogInner<C: CheckpointerImpl> {
    fetcher: Arc<C::CheckpointFetcher>,
    store: LruCacheStore<C::CheckpointStore>,
    #[allow(clippy::type_complexity)]
    tiles: Option<TileFetcher<LruCacheStore<TileFetchStore<C::CheckpointFetcher>>>>,
}
