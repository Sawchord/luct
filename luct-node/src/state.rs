use crate::conf::Config;
use axum::extract::State;
use otlsp_server::OtlspMetrics;
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone)]
pub(crate) struct NodeState(Arc<NodeStateInner>);

#[derive(Debug)]
struct NodeStateInner {
    config: Config,
    otlsp_urls: Vec<Url>,
    otlsp_metrics: OtlspMetrics,
}

impl NodeState {
    pub(crate) fn new(config: Config) -> eyre::Result<Self> {
        let urls = config.get_otlsp_urls()?;

        Ok(Self(Arc::new(NodeStateInner {
            config,
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

    pub(crate) fn otlsp_metrics(&self) -> State<OtlspMetrics> {
        State(self.0.otlsp_metrics.clone())
    }
}
