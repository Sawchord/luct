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

// #[derive(Debug, Clone, PartialEq, Eq, Default)]
// pub struct OtlspClientBuilder {
//     connection_timeout: Duration,
//     proxy_url: Option<Url>,
//     agent: Option<String>,
// }

// impl From<OtlspClientBuilder> for OtlspClientConfig {
//     fn from(builder: OtlspClientBuilder) -> Self {
//         Self {
//             connection_timeout: builder
//                 .connection_timeout
//                 .unwrap_or(Duration::from_secs(30)),
//             proxy_url: builder.proxy_url,
//             agent: builder.agent.unwrap_or("".to_string()),
//         }
//     }
// }

// impl OtlspClientBuilder {
//     pub fn build(self) -> OtlspClient {
//         let config: OtlspClientConfig = self.into();
//         OtlspClient {
//             fallback: ReqwestClient::new(&config.agent),
//             config: Arc::new(config),
//             connections: Arc::new(Mutex::new(HashMap::new())),
//         }
//     }

//     pub fn connection_timeout(mut self, timeout: Duration) -> Self {
//         self.connection_timeout = Some(timeout);
//         self
//     }

//     pub fn proxy_url(mut self, url: Url) -> Self {
//         self.proxy_url = Some(url);
//         self
//     }

//     pub fn agent(mut self, agent: String) -> Self {
//         self.agent = Some(agent);
//         self
//     }
// }
