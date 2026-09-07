use crate::checkpoint::log::CheckpointLog;
use luct_client::Client;
use luct_core::{store::SearchableStore, v1::LogId};
use std::{collections::BTreeMap, sync::Arc};

mod error;
mod log;

#[derive(Clone, Debug)]
pub struct CheckpointerConfig {}

/// Bundle trait for [`Checkpointer`]
pub trait CheckpointerImpl {
    /// The [`Client`] that is used to fetch the STH checkpoints and extension proofs from the logs
    type CheckpointFetcher: Client + Clone;
    /// The [`Store`](luct_core::store::Store) that stores the STH checkpoints in serialized format
    type CheckpointStore: SearchableStore<Key = u64, Value = String>;
}

/// A [`Checkpointer`] is a tool used to create and serve STH checkpoints
#[derive(Clone, Debug)]
pub struct Checkpointer<C: CheckpointerImpl> {
    config: Arc<CheckpointerConfig>,
    logs: BTreeMap<LogId, CheckpointLog<C>>,
    fetcher: Arc<C::CheckpointFetcher>,
    store: C::CheckpointStore,
}
