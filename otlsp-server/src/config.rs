use serde::{Deserialize, Serialize};
use url::Url;

/// Configure the OTLSP server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Config {
    /// Route at which to mount the proxy
    pub(crate) route: String,

    /// Endpoint
    pub(crate) endpoint: String,

    /// List of URLs, which the connection can be forwarded to
    pub(crate) enabled_urls: Vec<Url>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            route: String::from("/"),
            endpoint: String::from("0.0.0.0:3000"),
            enabled_urls: vec![Url::parse("https://127.0.0.1:8080").unwrap()],
        }
    }
}
