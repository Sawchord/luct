pub use crate::config::OtlspClientBuilder;
use crate::{config::OtlspClientConfig, connection::OtlspConnection};
use luct_client::{Client, ClientError, reqwest::ReqwestClient};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use url::{Host, Url};

mod config;
mod connection;

#[derive(Debug, Clone)]
pub struct OtlspClient {
    config: Arc<OtlspClientConfig>,
    connections: Arc<RwLock<HashMap<Host, Arc<Mutex<OtlspConnection>>>>>,
    fallback: ReqwestClient,
}

impl Client for OtlspClient {
    async fn get(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, Arc<String>), ClientError> {
        if self.config.proxy_url.is_some() {
            return self.fallback.get(url, params).await;
        };

        let connection = self.get_connection(url).await?;
        let request = connection.lock().unwrap().get(url, params)?;
        request.await.map(|(status, bytes)| {
            (
                status,
                Arc::new(String::from_utf8_lossy(&bytes).to_string()),
            )
        })
    }

    async fn get_bin(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, Arc<Vec<u8>>), ClientError> {
        if self.config.proxy_url.is_some() {
            return self.fallback.get_bin(url, params).await;
        };

        let connection = self.get_connection(url).await?;
        let request = connection.lock().unwrap().get(url, params)?;
        request
            .await
            .map(|(status, bytes)| (status, Arc::new(bytes)))
    }
}

impl OtlspClient {
    async fn get_connection(&self, url: &Url) -> Result<Arc<Mutex<OtlspConnection>>, ClientError> {
        let Some(domain) = url.host() else {
            return Err(ClientError::ConnectionError("Invalid url".to_string()));
        };

        let conns = self.connections.read().unwrap();
        if let Some(connection) = conns.get(&domain.to_owned())
            && !connection.lock().unwrap().has_timed_out()
        {
            return Ok(connection.clone());
        }
        todo!()
    }
}
