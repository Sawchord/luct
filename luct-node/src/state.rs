use crate::{
    NodeCheckpointerImpl, USER_AGENT,
    checkpoint::{Checkpointer, config::CheckpointerConfig, metrics::CheckpointerMetrics},
    conf::Config,
    otlsp::get_otlsp_urls,
};
use axum::extract::State;
use luct_client::{deduplication::RequestDeduplicationClient, reqwest::ReqwestClient};
use luct_store::FilesystemStore;
use otlsp_server::{OtlspConfig, OtlspMetrics, OtlspState};
use std::{path::PathBuf, sync::Arc, time::Duration};
use url::Url;

#[derive(Debug, Clone)]
pub(crate) struct NodeState(Arc<NodeStateInner>);

#[derive(Debug)]
struct NodeStateInner {
    config: Arc<Config>,
    otlsp_state: OtlspState,
    otlsp_urls: Vec<Url>,
    checkpointer: Checkpointer<NodeCheckpointerImpl>,
}

impl NodeState {
    pub(crate) async fn new(config: Config) -> eyre::Result<Self> {
        let logs = config.get_active_logs()?;
        let urls = get_otlsp_urls(&logs);

        let otlsp_config = Arc::new(OtlspConfig {
            buffer_size: config.otlsp_packet_buffer_size.unwrap_or(100),
        });

        let mut cp_builder = CheckpointerConfig::builder();
        config
            .minimal_checkpoint_interval
            .map(|val| cp_builder.minimal_update_interval(Duration::from_secs(val)));
        config
            .maximal_checkpoint_interval
            .map(|val| cp_builder.maximal_update_interval(Duration::from_secs(val)));

        let fetcher = RequestDeduplicationClient::new(ReqwestClient::new(USER_AGENT));

        let checkpoint_dir = config
            .checkpoint_store_path
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::home_dir()
                    .expect("Home directory not test")
                    .join(".luct")
            });

        let mut checkpointer =
            Checkpointer::new(cp_builder.build()?, CheckpointerMetrics::default(), fetcher);

        for log in logs {
            let name = log.description();
            checkpointer
                .add_log(
                    &log,
                    FilesystemStore::new(checkpoint_dir.join("sth").join(name)),
                )
                .await?;
        }

        Ok(Self(Arc::new(NodeStateInner {
            config: Arc::new(config),
            otlsp_urls: urls,
            otlsp_state: OtlspState {
                config: otlsp_config,
                metrics: OtlspMetrics::default(),
            },
            checkpointer,
        })))
    }

    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

    pub(crate) fn otlsp_urls(&self) -> &[Url] {
        &self.0.otlsp_urls
    }

    pub(crate) fn otlsp_state(&self) -> State<OtlspState> {
        State(self.0.otlsp_state.clone())
    }
}
