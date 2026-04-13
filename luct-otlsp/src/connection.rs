use std::sync::Arc;

use crate::config::OtlspClientConfig;
use hyper::{Request, Response, body::Incoming, client::conn::http1::SendRequest};
use luct_client::ClientError;
use otlsp_client::OtlspClientBuilder;
use url::Url;
use web_time::Instant;

// TODO: Use OTLSP errors here and do the conversion one level up

#[derive(Debug)]
pub(crate) struct OtlspConnection {
    last_access: Instant,
    config: Arc<OtlspClientConfig>,
    host: String,
    sender: SendRequest<String>,
}

impl OtlspConnection {
    pub(crate) async fn new(config: Arc<OtlspClientConfig>, url: Url) -> Result<Self, ClientError> {
        let Some(proxy_url) = &config.proxy_url else {
            return Err(ClientError::ConnectionError("Proxy Url unset".to_string()));
        };

        let Some(host) = url.host_str() else {
            return Err(ClientError::ConnectionError(
                "Invalid destination url".to_string(),
            ));
        };

        let sender = OtlspClientBuilder::new(proxy_url.clone())
            .with_webpki_roots()
            .handshake(url.clone())
            .await
            .map_err(|err| ClientError::ConnectionError(err.to_string()))?;

        tracing::debug!(
            "Created new proxy connection to {} via proxy {}",
            url,
            proxy_url
        );

        Ok(Self {
            last_access: Instant::now(),
            config,
            host: host.to_string(),
            sender,
        })
    }

    pub(crate) fn get(
        &mut self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> Result<impl Future<Output = Result<Response<Incoming>, hyper::Error>>, ClientError> {
        if Some(self.host.as_str()) != url.host_str() {
            return Err(ClientError::ConnectionError(
                "Url mismatch with the connection".to_string(),
            ));
        }

        let req = Request::builder()
            // TODO: Add headers for host, agent and params
            .uri("url")
            .method("GET")
            .body("".to_string())
            .map_err(|err| ClientError::ConnectionError(err.to_string()))?;

        Ok(self.sender.send_request(req))
    }
}
