use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration values for the [`Checkpointer`](crate::checkpoint::Checkpointer)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Builder)]
#[builder(setter(into))]
pub struct CheckpointerConfig {
    /// Size of the checkpoint cache (per log)
    ///
    /// These are the values that are supposed to be helt in cache to be
    /// served directly
    #[builder(default = "1000")]
    pub(crate) checkpoint_cache_size: usize,

    /// Size of the tile cache (per log)
    #[builder(default = "1000")]
    pub(crate) tile_cache_size: usize,

    /// Minimal update interval
    ///
    /// The minimal interval that the checkpointer should wait
    /// before trying to make a new update
    #[builder(default = "Duration::from_secs(60 * 60 * 8)")]
    pub(crate) minimal_update_interval: Duration,

    /// Maximal update interval
    ///
    /// The maximal interval that the checkpointer should wait
    /// before trying to make a new update
    #[builder(default = "Duration::from_secs(60 * 60 * 24)")]
    pub(crate) maximal_update_interval: Duration,
}

impl CheckpointerConfig {
    pub fn builder() -> CheckpointerConfigBuilder {
        CheckpointerConfigBuilder::default()
    }
}
