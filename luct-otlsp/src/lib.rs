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
        let Some(proxy_url) = &self.config.proxy_url else {
            return self.fallback.get(url, params).await;
        };

        todo!()
    }

    async fn get_bin(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, Arc<Vec<u8>>), ClientError> {
        let Some(proxy_url) = &self.config.proxy_url else {
            return self.fallback.get_bin(url, params).await;
        };

        todo!()
    }
}

impl OtlspClient {
    fn get_connection(&self, url: &Url) -> Result<Arc<Mutex<OtlspConnection>>, ClientError> {
        todo!()
    }
}
