use crate::OtlspClient;
use derive_builder::Builder;
use luct_client::reqwest::ReqwestClient;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use url::Url;
use web_time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
pub struct OtlspClientConfig {
    #[builder(default = "Duration::from_secs(30)")]
    pub(crate) connection_timeout: Duration,
    #[builder(default = "None")]
    pub(crate) proxy_url: Option<Url>,
    pub(crate) agent: String,
}

impl OtlspClient {
    pub fn new(config: OtlspClientConfig) -> Self {
        OtlspClient {
            fallback: ReqwestClient::new(&config.agent),
            config: Arc::new(config),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl OtlspClientConfig {
    pub fn builder() -> OtlspClientConfigBuilder {
        OtlspClientConfigBuilder::default()
    }
}
