use crate::conf::Config;
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone)]
pub(crate) struct NodeState(Arc<NodeStateInner>);

#[derive(Debug)]
struct NodeStateInner {
    config: Config,
    otlsp_urls: Vec<Url>,
}

impl NodeState {
    pub(crate) fn new(config: Config) -> eyre::Result<Self> {
        let urls = config.get_otlsp_urls()?;

        Ok(Self(Arc::new(NodeStateInner {
            config,
            otlsp_urls: urls,
        })))
    }

    pub(crate) fn config(&self) -> &Config {
        &self.0.config
    }

    pub(crate) fn otlsp_urls(&self) -> &[Url] {
        &self.0.otlsp_urls
    }
}
