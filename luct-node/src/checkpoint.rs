use crate::{
    NodeCheckpointerImpl,
    checkpoint::{
        config::CheckpointerConfig, error::CheckpointerError, log::CheckpointLog,
        metrics::CheckpointerMetrics,
    },
    state::NodeState,
};
use axum::{
    extract::{Path, State},
    response::Response,
};
use axum_macros::debug_handler;
use luct_client::Client;
use luct_core::{CtLog, LogId, store::SearchableStore};
use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

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

#[debug_handler]
pub(crate) async fn handle_toc_request(state: State<NodeState>, log_id: Path<String>) -> Response {
    let checkpointer = match state.0.try_get_log(&log_id) {
        Ok(cp) => cp,
        Err(response) => return *response,
    };

    let toc = checkpointer.serve_toc();
    Response::builder().status(200).body(toc.into()).unwrap()
}

#[debug_handler]
pub(crate) async fn handle_checkpoint_request(
    state: State<NodeState>,
    params: Path<(String, u64)>,
) -> Response {
    let checkpointer = match state.0.try_get_log(&params.0.0) {
        Ok(cp) => cp,
        Err(response) => return *response,
    };

    let Some(cp) = checkpointer.serve_checkpoint(params.0.1).await else {
        return Response::builder()
            .status(404)
            .body("Could not find checkpoint with that tree_size".into())
            .unwrap();
    };

    Response::builder().status(200).body(cp.into()).unwrap()
}

impl NodeState {
    fn try_get_log(
        &self,
        log_id: &str,
    ) -> Result<&CheckpointLog<NodeCheckpointerImpl>, Box<Response>> {
        let log_id = luct_core::v1::LogId::try_from(log_id).map_err(|_| {
            Response::builder()
                .status(400)
                .body("Failed to parse log id".into())
                .unwrap()
        })?;
        let log_id = LogId::V1(log_id);

        let log = self.0.checkpointer.logs.get(&log_id).ok_or_else(|| {
            Response::builder()
                .status(404)
                .body("Unknown log id".into())
                .unwrap()
        })?;

        Ok(log)
    }

    pub(crate) fn schedule_updates(&self) {
        for log in self.0.checkpointer.logs.values() {
            let log = log.clone();
            tokio::spawn(async move {
                loop {
                    let next_update = log.next_update(&mut rand::rng());
                    tracing::info!(
                        "Scheduled update for log {} at {:?}",
                        log.name(),
                        next_update
                    );

                    let time_to_update = next_update
                        .duration_since(SystemTime::now())
                        .unwrap_or(Duration::default());
                    tokio::time::sleep(time_to_update).await;

                    match log.update_sth().await {
                        Ok(()) => {
                            tracing::info!("Updated log {} ", log.name())
                        }
                        Err(err) => {
                            tracing::warn!(
                                "Failed to update checkpoint of log {}: {:?}",
                                log.name(),
                                err
                            );
                            tokio::time::sleep(Duration::from_secs(60)).await;
                        }
                    }
                }
            });
        }
    }
}
