use crate::conf::Config;
use axum::extract::State;
use otlsp_server::{OtlspConfig, OtlspMetrics, OtlspState};
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone)]
pub(crate) struct NodeState(Arc<NodeStateInner>);

#[derive(Debug)]
struct NodeStateInner {
    config: Arc<Config>,
    otlsp_config: Arc<OtlspConfig>,
    otlsp_urls: Vec<Url>,
    otlsp_metrics: OtlspMetrics,
}

impl NodeState {
    pub(crate) fn new(config: Config) -> eyre::Result<Self> {
        let urls = config.get_otlsp_urls()?;

        let otlsp_config = Arc::new(OtlspConfig {
            buffer_size: config.otlsp_packet_buffer_size.unwrap_or(100),
        });

        Ok(Self(Arc::new(NodeStateInner {
            config: Arc::new(config),
            otlsp_config,
            otlsp_urls: urls,
            otlsp_metrics: OtlspMetrics::default(),
        })))
    }

    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

    pub(crate) fn otlsp_urls(&self) -> &[Url] {
        &self.0.otlsp_urls
    }

    pub(crate) fn otlsp_state(&self) -> State<OtlspState> {
        State(OtlspState {
            config: self.0.otlsp_config.clone(),
            metrics: self.0.otlsp_metrics.clone(),
        })
    }
}
