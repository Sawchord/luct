use crate::config::OtlspClientConfig;
use hyper::client::conn::http1::SendRequest;
use luct_client::ClientError;
use otlsp_client::OtlspClientBuilder;
use url::Url;
use web_time::Instant;

#[derive(Debug)]
pub(crate) struct OtlspConnection {
    last_access: Instant,
    host: String,
    sender: SendRequest<String>,
}

impl OtlspConnection {
    pub(crate) async fn new(config: &OtlspClientConfig, url: Url) -> Result<Self, ClientError> {
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
            host: host.to_string(),
            sender,
        })
    }
}
