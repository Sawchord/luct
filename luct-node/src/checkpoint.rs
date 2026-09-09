use crate::checkpoint::{
    config::CheckpointerConfig, error::CheckpointerError, log::CheckpointLog,
    metrics::CheckpointerMetrics,
};
use luct_client::Client;
use luct_core::{CtLog, LogId, store::SearchableStore};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) mod config;
mod error;
mod log;
pub(crate) mod metrics;

/// Bundle trait for [`Checkpointer`]
pub trait CheckpointerImpl {
    /// The [`Client`] that is used to fetch the STH checkpoints and extension proofs from the logs
    type CheckpointFetcher: Client + Clone;
    /// The [`Store`](luct_core::store::Store) that stores the STH checkpoints in serialized format
    type CheckpointStore: SearchableStore<Key = u64, Value = String>;
}

/// A [`Checkpointer`] is a tool used to create and serve STH checkpoints
#[derive(Debug)]
pub struct Checkpointer<C: CheckpointerImpl> {
    config: Arc<CheckpointerConfig>,
    metrics: CheckpointerMetrics,
    logs: BTreeMap<LogId, CheckpointLog<C>>,
    fetcher: C::CheckpointFetcher,
}

impl<C: CheckpointerImpl> Checkpointer<C> {
    pub fn new(
        config: CheckpointerConfig,
        metrics: CheckpointerMetrics,
        fetcher: C::CheckpointFetcher,
    ) -> Self {
        Self {
            config: Arc::new(config),
            metrics,
            logs: BTreeMap::new(),
            fetcher,
        }
    }

    pub async fn add_log(
        &mut self,
        log: &CtLog,
        store: C::CheckpointStore,
    ) -> Result<&mut Self, CheckpointerError> {
        let new_log = CheckpointLog::new(
            log,
            self.config.clone(),
            self.metrics.clone(),
            store,
            self.fetcher.clone(),
        )
        .await?;
        let log_id = log.log_id().clone();

        self.logs.insert(log_id, new_log);
        Ok(self)
    }
}
