use derive_builder::Builder;
use serde::{Deserialize, Serialize};

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
}

impl CheckpointerConfig {
    pub fn builder() -> CheckpointerConfigBuilder {
        CheckpointerConfigBuilder::default()
    }
}
