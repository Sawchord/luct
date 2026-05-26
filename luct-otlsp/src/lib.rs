#![forbid(unsafe_code)]

pub use crate::config::OtlspClientBuilder;
use crate::{config::OtlspClientConfig, connection::OtlspConnection};
use futures::lock::Mutex as FutMutex;
use luct_client::{Client, ClientError, reqwest::ReqwestClient};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};
use url::{Host, Url};

mod config;
mod connection;

#[derive(Debug, Clone)]
pub struct OtlspClient {
    config: Arc<OtlspClientConfig>,
    connections: Arc<Mutex<HashMap<Host, Arc<FutMutex<OtlspConnection>>>>>,
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
        let (status, response) = Self::request(&connection, url, params).await?;
        Ok((
            status,
            Arc::new(String::from_utf8_lossy(&response).to_string()),
        ))
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
        let (status, response) = Self::request(&connection, url, params).await?;
        Ok((status, Arc::new(response)))
    }
}

impl OtlspClient {
    // NOTE: We have one await statement where we need to do this.
    // The await should actually return immidiately. See comments in the function
    #[allow(clippy::await_holding_lock)]
    async fn get_connection(
        &self,
        url: &Url,
    ) -> Result<Arc<FutMutex<OtlspConnection>>, ClientError> {
        let Some(domain) = url.host() else {
            return Err(ClientError::ConnectionError("Invalid url".to_string()));
        };
        let domain = domain.to_owned();

        // NOTE: In order to prevent data races which lead to unnecessary connections:
        let connections = self.connections.lock().unwrap();

        // 1. We check if there is an existing connection in the state
        let connection = match connections.get(&domain) {
            None => {
                // 2a. If not, we create a new connection and insert it in the state
                new_connection(connections, self.config.clone(), url.clone(), domain).await?
            }
            Some(connection) => {
                // 2b. If we already have a connection, we clone it and let go of the state lock
                let connection = connection.clone();
                drop(connections);
                dbg!("Found existing connection");

                // 3b. If the connection has timed out, we remove it and call get_connection again
                // We need to take the connections mutex again
                // NOTE: Holding the existing mutex will lead to a deadlock
                if connection.lock().await.has_timed_out() {
                    tracing::debug!(
                        "Connection timed out, establishing fresh connection: {}",
                        url.as_str()
                    );
                    let mut connections = self.connections.lock().unwrap();
                    connections.remove(&domain);
                    new_connection(connections, self.config.clone(), url.clone(), domain).await?
                } else {
                    connection
                }
            }
        };

        async fn new_connection(
            mut connections: MutexGuard<'_, HashMap<Host, Arc<FutMutex<OtlspConnection>>>>,
            config: Arc<OtlspClientConfig>,
            url: Url,
            domain: Host,
        ) -> Result<Arc<FutMutex<OtlspConnection>>, ClientError> {
            let connection = Arc::new(FutMutex::new(OtlspConnection::new(config, url)));
            connections.insert(domain, connection.clone());

            // 3. We then take out a lock on the new connection, this should never block since we have still exclusive access
            let mut connection_lock = connection.lock().await;

            // 4. We release the lock on the connection state
            drop(connections);

            // 5. Now we actually establish the handshake. New requests can already see the connection but will
            // wait on the mutex, until this returns
            connection_lock
                .establish()
                .await
                .map_err(|err| ClientError::ConnectionErrorStd(Arc::new(err)))?;

            // 6. Return the now established connection. This will drop connection_lock and allow other requesters to use is
            drop(connection_lock);
            Ok(connection)
        }

        Ok(connection)
    }

    async fn request(
        connection: &Arc<FutMutex<OtlspConnection>>,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, Vec<u8>), ClientError> {
        let response = connection
            .lock()
            .await
            .get_async(url, params)
            .await
            .map_err(|err| ClientError::ConnectionErrorStd(Arc::new(err)))?;
        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use crate::OtlspClient;
    use luct_client::CtClient;
    use luct_core::{CtLogConfig, tiling::TileId, tree::NodeKey};
    use luct_test::utils::test_tracing;
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
    #[ignore = "Makes an OTSLP call, for manual testing only"]
    async fn get_checkpoint_wasm() {
        get_checkpoint().await
    }

    #[tokio::test]
    #[ignore = "Makes an OTSLP call, for manual testing only"]
    async fn get_checkpoint_native() {
        get_checkpoint().await
    }

    async fn get_checkpoint() {
        test_tracing();

        let client = get_client(SYC2027H2);
        let _ = client.get_checkpoint().await.unwrap();
    }

    #[wasm_bindgen_test]
    #[ignore = "Makes an OTSLP call, for manual testing only"]
    async fn get_tile_wasm() {
        get_tile().await
    }

    #[tokio::test]
    #[ignore = "Makes an OTSLP call, for manual testing only"]
    async fn get_tile_native() {
        get_tile().await
    }

    async fn get_tile() {
        test_tracing();

        let client = get_client(SYC2027H2);

        let _ = client
            .get_tile(TileId::from_node_key(&NodeKey::leaf(1), 1000).unwrap())
            .await
            .unwrap();
        let _ = client.get_checkpoint().await.unwrap();
    }

    // TODO: Test with parameters

    fn get_client(log: &str) -> CtClient<OtlspClient> {
        let config: CtLogConfig = serde_json::from_str(log).unwrap();
        let client = OtlspClient::builder()
            .proxy_url(Url::parse("https://node.luct.dev/otlsp").unwrap())
            .agent("luct-otlsp-test".to_string())
            .build();
        CtClient::new(config, client)
    }
}
