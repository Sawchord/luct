use config::{Config as Conf, Environment, File};
use eyre::Context;
use luct_core::{CtLog, log_list::v3::LogList};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Config {
    /// Endpoint address
    pub(crate) endpoint_addr: String,

    /// Path to the log list to use in luct-node
    pub(crate) log_list: String,

    /// Path at which to serve the oblivious TLS proxy
    pub(crate) otlsp_path: Option<String>,

    /// Time (in seconds) until a connection is terminated
    pub(crate) otlsp_timeout_seconds: Option<u64>,

    /// Number of in-flight packets of an OTLSP channel (per direction)
    pub(crate) otlsp_packet_buffer_size: Option<usize>,

    /// Path at which to serve the checkpointer
    pub(crate) checkpoint_path: Option<String>,

    /// File path at which the checkpoints will be stored
    pub(crate) checkpoint_store_path: Option<String>,

    /// Minimal interval (in seconds) between checkpoints
    pub(crate) minimal_checkpoint_interval: Option<u64>,

    /// Maximal interval (in seconds) between checkpoints
    pub(crate) maximal_checkpoint_interval: Option<u64>,

    /// Path at which to serve the metrics endpoint
    pub(crate) metrics_path: Option<String>,
}

impl Config {
    pub(crate) fn parse() -> eyre::Result<Self> {
        let config = Conf::builder()
            .add_source(File::from(PathBuf::from("/etc/luct/luct.toml")).required(false))
            .add_source(
                File::from(
                    std::env::home_dir()
                        .expect("Home directory not set")
                        .join(".luct/luct.toml"),
                )
                .required(false),
            )
            .add_source(File::from(PathBuf::from("luct.toml")).required(false))
            .add_source(Environment::with_prefix("LUCT"))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    pub(crate) fn get_active_logs(&self) -> eyre::Result<Vec<CtLog>> {
        let logs = std::fs::read_to_string(&self.log_list)
            .with_context(|| format! {"Could not find log list file at {}", self.log_list})?;
        let logs: LogList =
            serde_json::from_str(&logs).with_context(|| "Failed to parse log list")?;
        let logs = logs.currently_active_logs();
        tracing::info!("Imported {} logs", logs.len());

        Ok(logs)
    }
}
