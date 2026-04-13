use crate::OtlspClient;
use luct_client::reqwest::ReqwestClient;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use url::Url;
use web_time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OtlspClientConfig {
    pub(crate) connection_timeout: Duration,
    pub(crate) proxy_url: Option<Url>,
    pub(crate) agent: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OtlspClientBuilder {
    connection_timeout: Option<Duration>,
    proxy_url: Option<Url>,
    agent: Option<String>,
}

impl OtlspClient {
    pub fn builder() -> OtlspClientBuilder {
        OtlspClientBuilder::default()
    }
}

impl From<OtlspClientBuilder> for OtlspClientConfig {
    fn from(builder: OtlspClientBuilder) -> Self {
        Self {
            connection_timeout: builder
                .connection_timeout
                .unwrap_or(Duration::from_secs(30)),
            proxy_url: builder.proxy_url,
            agent: builder.agent.unwrap_or("".to_string()),
        }
    }
}

impl OtlspClientBuilder {
    pub fn build(self) -> OtlspClient {
        let config: OtlspClientConfig = self.into();
        OtlspClient {
            fallback: ReqwestClient::new(&config.agent),
            config: Arc::new(config),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    pub fn proxy_url(mut self, url: Url) -> Self {
        self.proxy_url = Some(url);
        self
    }

    pub fn agent(mut self, agent: String) -> Self {
        self.agent = Some(agent);
        self
    }
}
