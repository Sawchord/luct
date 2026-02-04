//! Implementation of the [`Client`] trait using [`reqwest`]

use crate::{Client, ClientError};
use reqwest::Response;
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct ReqwestClient {
    client: reqwest::Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        Self {
            // TODO: Make the user agent setable
            client: reqwest::Client::new(),
        }
    }
}

impl Client for ReqwestClient {
    #[tracing::instrument(level = "trace")]
    async fn get(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, Arc<String>), ClientError> {
        let response = self.request(url, params).await?;
        let status = response.status().as_u16();
        let data = response
            .text()
            .await
            .map_err(|err| ClientError::ConnectionError(err.to_string()))?;

        Ok((status, Arc::new(data)))
    }

    #[tracing::instrument(level = "trace")]
    async fn get_bin(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, Arc<Vec<u8>>), ClientError> {
        let response = self.request(url, params).await?;
        let status = response.status().as_u16();
        let data = response
            .bytes()
            .await
            .map_err(|err| ClientError::ConnectionError(err.to_string()))?;

        Ok((status, Arc::new(data.to_vec())))
    }
}

impl ReqwestClient {
    async fn request(&self, url: &Url, params: &[(&str, &str)]) -> Result<Response, ClientError> {
        self.client
            .get(url.clone())
            .query(params)
            .send()
            .await
            .map_err(|err| ClientError::ConnectionError(err.to_string()))
    }
}
