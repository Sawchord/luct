use prometheus::{
    IntCounterVec, IntGaugeVec, Opts, Registry, default_registry,
    register_int_counter_vec_with_registry, register_int_gauge_vec_with_registry,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CheckpointerMetrics {
    last_update: IntGaugeVec,
    toc_served: IntCounterVec,
    checkpoint_served: IntCounterVec,
}

impl CheckpointerMetrics {
    pub fn new_with_registry(registry: &Registry) -> Self {
        Self {
            last_update: register_int_gauge_vec_with_registry!(
                Opts::new(
                    "last_update",
                    "Timestamp (in seconds) when the STH was updated"
                ),
                &["log_name"],
                registry
            )
            .unwrap(),
            toc_served: register_int_counter_vec_with_registry!(
                Opts::new(
                    "toc_served",
                    "Number of times that the table of contents of a checkpointer was served"
                ),
                &["log_name"],
                registry
            )
            .unwrap(),
            checkpoint_served: register_int_counter_vec_with_registry!(
                Opts::new(
                    "checkpoint_served",
                    "Number of times that a checkpoint was served"
                ),
                &["log_name"],
                registry
            )
            .unwrap(),
        }
    }

    pub(crate) fn set_last_update(&self, name: &str, time: SystemTime) {
        let time = UNIX_EPOCH.duration_since(time).unwrap().as_secs();
        self.last_update.with_label_values(&[name]).set(time as i64);
    }

    pub(crate) fn toc_served(&self, name: &str) {
        self.toc_served.with_label_values(&[name]).inc();
    }

    pub(crate) fn checkpoint_served(&self, name: &str) {
        self.checkpoint_served.with_label_values(&[name]).inc();
    }
}

impl Default for CheckpointerMetrics {
    fn default() -> Self {
        Self::new_with_registry(default_registry())
    }
}
