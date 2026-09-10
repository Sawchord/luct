use crate::{conf::Config, otlsp::get_otlsp_urls};
use axum::extract::State;
use otlsp_server::{OtlspConfig, OtlspMetrics, OtlspState};
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone)]
pub(crate) struct NodeState(Arc<NodeStateInner>);

#[derive(Debug)]
struct NodeStateInner {
    config: Arc<Config>,
    otlsp_state: OtlspState,
    otlsp_urls: Vec<Url>,
}

impl NodeState {
    pub(crate) fn new(config: Config) -> eyre::Result<Self> {
        let logs = config.get_active_logs()?;
        let urls = get_otlsp_urls(&logs);

        let otlsp_config = Arc::new(OtlspConfig {
            buffer_size: config.otlsp_packet_buffer_size.unwrap_or(100),
        });

        Ok(Self(Arc::new(NodeStateInner {
            config: Arc::new(config),
            otlsp_urls: urls,
            otlsp_state: OtlspState {
                config: otlsp_config,
                metrics: OtlspMetrics::default(),
            },
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
