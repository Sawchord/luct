pub use crate::config::OtlspClientBuilder;
use crate::config::OtlspClientConfig;
use hyper::client::conn::http1::SendRequest;
use luct_client::{Client, ClientError, reqwest::ReqwestClient};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use url::{Host, Url};
use web_time::Instant;

mod config;
mod connection;

#[derive(Debug)]
struct OtlspConnection {
    time: Instant,
    sender: SendRequest<String>,
}

#[derive(Debug, Clone)]
pub struct OtlspClient {
    config: OtlspClientConfig,
    connections: HashMap<Host, Arc<Mutex<OtlspConnection>>>,
    fallback: ReqwestClient,
}

impl Client for OtlspClient {
    async fn get(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, Arc<String>), ClientError> {
        todo!()
    }

    async fn get_bin(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, Arc<Vec<u8>>), ClientError> {
        todo!()
    }
}
