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
    async fn get(&self, url: &Url, params: &[(&str, &str)]) -> Result<(u16, String), ClientError> {
        let response = self
            .client
            .get(url.clone())
            .query(params)
            .send()
            .await
            .map_err(|err| ClientError::ConnectionError(err.to_string()))?;

        let status = response.status().as_u16();
        let data = response
            .text()
            .await
            .map_err(|err| ClientError::ConnectionError(err.to_string()))?;

        Ok((status, data))
    }
}
