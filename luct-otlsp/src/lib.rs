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
        let domain = domain.to_owned();

        if let Some(connection) = self.connections.read().unwrap().get(&domain)
            && !connection.lock().unwrap().has_timed_out()
        {
            return Ok(connection.clone());
        }

        let connection = OtlspConnection::new(self.config.clone(), url.clone()).await?;
        let connection = Arc::new(Mutex::new(connection));
        self.connections
            .write()
            .unwrap()
            .insert(domain, connection.clone());

        Ok(connection)
    }
}

#[cfg(test)]
mod test {
    use crate::OtlspClient;
    use luct_client::CtClient;
    use luct_core::CtLogConfig;
    use tracing::Level;
    use tracing_wasm::{ConsoleConfig, WASMLayerConfigBuilder};
    use url::Url;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    const ARCHE2026H1: &str = "{
          \"description\": \"Google 'Arche2026h1' log\",
          \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEZ+3YKoZTMruov4cmlImbk4MckBNzEdCyMuHlwGgJ8BUrzFLlR5U0619xDDXIXespkpBgCNVQAkhMTTXakM6KMg==\",
          \"url\": \"https://arche2026h1.staging.ct.transparency.dev/\",
          \"tile_url\": \"https://storage.googleapis.com/static-ct-staging-arche2026h1-bucket/\",
          \"mmd\": 60
        }";

    const SYC2027H2: &str = "{
          \"description\": \"Let's Encrypt 'Sycamore2027h2'\",
          \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEK+2zy2UWRMIyC2jU46+rj8UsyMjLsQIr1Y/6ClbdpWGthUb8y3Maf4zfAZTWW+AH9wAWPLRL5vmtz7Zkh2f2nA==\",
          \"url\": \"https://log.sycamore.ct.letsencrypt.org/2027h2/\",
          \"tile_url\": \"https://mon.sycamore.ct.letsencrypt.org/2027h2/\",
          \"mmd\": 60
        }";

    #[wasm_bindgen_test]
    async fn get_checkpoint() {
        tracing();

        let client = get_client(SYC2027H2);
        let _ = client.get_checkpoint().await.unwrap();
    }

    // TODO: Test getting a tile
    // TODO: Test with parameters

    fn tracing() {
        tracing_wasm::set_as_global_default_with_config(
            WASMLayerConfigBuilder::default()
                .set_max_level(Level::TRACE)
                .set_console_config(ConsoleConfig::ReportWithoutConsoleColor)
                .build(),
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
