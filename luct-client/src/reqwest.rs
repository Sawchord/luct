//! Implementation of the [`Client`] trait using [`reqwest`]

use crate::{Client, ClientError};
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Client for ReqwestClient {
    async fn get(&self, url: &Url, params: &[(&str, &str)]) -> Result<String, ClientError> {
        dbg!(&url);
        self.client
            .get(url.clone())
            .form(params)
            .send()
            .await
            .map_err(|err| ClientError::ConnectionError(err.to_string()))?
            .text()
            .await
            .map_err(|err| ClientError::ConnectionError(err.to_string()))
    }
}
