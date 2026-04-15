pub use crate::config::OtlspClientBuilder;
use crate::{config::OtlspClientConfig, connection::OtlspConnection};
use futures::TryFutureExt;
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
        if self.config.proxy_url.is_none() {
            return self.fallback.get(url, params).await;
        };

        let connection = self.get_connection(url).await?;
        let request = Self::request(connection, url, params)?;

        // NOTE: The lock on connection is already dropped here
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
        if self.config.proxy_url.is_none() {
            return self.fallback.get_bin(url, params).await;
        };

        let connection = self.get_connection(url).await?;
        let request = Self::request(connection, url, params)?;

        // NOTE: The lock on connection is already dropped here
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
        let domain = domain.to_owned();

        if let Some(connection) = self.connections.read().unwrap().get(&domain)
            && !connection.lock().unwrap().has_timed_out()
        {
            tracing::trace!("Reusing existing connection to {}", url);
            return Ok(connection.clone());
        }

        tracing::trace!("Establishing new connection to {}", url);
        let connection = OtlspConnection::new(self.config.clone(), url.clone())
            .await
            .map_err(|err| ClientError::ConnectionErrorStd(Arc::new(err)))?;
        let connection = Arc::new(Mutex::new(connection));
        self.connections
            .write()
            .unwrap()
            .insert(domain, connection.clone());

        Ok(connection)
    }

    fn request(
        connection: Arc<Mutex<OtlspConnection>>,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<impl Future<Output = Result<(u16, Vec<u8>), ClientError>>, ClientError> {
        connection
            .lock()
            .unwrap()
            .get(url, params)
            .map(|fut| fut.map_err(|err| ClientError::ConnectionErrorStd(Arc::new(err))))
            .map_err(|err| ClientError::ConnectionErrorStd(Arc::new(err)))
    }
}

#[cfg(test)]
mod test {
    use crate::OtlspClient;
    use luct_client::CtClient;
    use luct_core::{CtLogConfig, tiling::TileId, tree::NodeKey};
    use tracing::Level;
    use tracing_subscriber::{Registry, layer::SubscriberExt};
    use tracing_wasm::{ConsoleConfig, WASMLayer, WASMLayerConfigBuilder};
    use url::Url;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    const SYC2027H2: &str = "{
          \"description\": \"Let's Encrypt 'Sycamore2027h2'\",
          \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEK+2zy2UWRMIyC2jU46+rj8UsyMjLsQIr1Y/6ClbdpWGthUb8y3Maf4zfAZTWW+AH9wAWPLRL5vmtz7Zkh2f2nA==\",
          \"url\": \"https://log.sycamore.ct.letsencrypt.org/2027h2/\",
          \"tile_url\": \"https://mon.sycamore.ct.letsencrypt.org/2027h2/\",
          \"mmd\": 60
        }";

    const _SOLERA2027H1: &str = "{
          \"description\": \"Google 'Solera2027h1' log'\",
          \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEKLDCw61qHAIp9mt8+QBx92gqIAhp3QhqS6h+bogJAe/FwaCSxMyb2H5xAO2SArXJnzm5vdBGifOcS5SXRZMP9Q==\",
          \"url\": \"https://ct.googleapis.com/logs/eu1/solera2027h1/\",
          \"mmd\": 86400
        }";

    #[wasm_bindgen_test]
    async fn get_checkpoint() {
        tracing();

        let client = get_client(SYC2027H2);
        let _ = client.get_checkpoint().await.unwrap();
    }

    #[wasm_bindgen_test]
    async fn get_tile() {
        tracing();

        let client = get_client(SYC2027H2);

        let _ = client
            .get_tile(TileId::from_node_key(&NodeKey::leaf(1), 1000).unwrap())
            .await
            .unwrap();
        let _ = client.get_checkpoint().await.unwrap();
    }

    // TODO: Test with parameters

    fn tracing() {
        let _ = tracing::subscriber::set_global_default(
            Registry::default().with(WASMLayer::new(
                WASMLayerConfigBuilder::default()
                    .set_max_level(Level::TRACE)
                    .set_console_config(ConsoleConfig::ReportWithoutConsoleColor)
                    .build(),
            )),
        );
    }

    fn get_client(log: &str) -> CtClient<OtlspClient> {
        let config: CtLogConfig = serde_json::from_str(log).unwrap();
        let client = OtlspClient::builder()
            .proxy_url(Url::parse("https://node.luct.dev/otlsp").unwrap())
            .agent("luct-otlsp-test".to_string())
            .build();
        CtClient::new(config, client)
    }
}
