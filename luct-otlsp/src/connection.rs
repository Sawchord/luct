use std::sync::Arc;

use crate::config::OtlspClientConfig;
use futures::FutureExt;
use http_body_util::BodyExt;
use hyper::{Request, client::conn::http1::SendRequest};
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
    ) -> Result<impl Future<Output = Result<(u16, Vec<u8>), ClientError>> + use<>, ClientError>
    {
        if Some(self.host.as_str()) != url.host_str() {
            return Err(ClientError::ConnectionError(
                "Url mismatch with the connection".to_string(),
            ));
        }

        let request = Request::builder()
            // TODO: Add headers for host, agent and params
            .uri("url")
            .method("GET")
            .body("".to_string())
            .map_err(|err| ClientError::ConnectionError(err.to_string()))?;

        // NOTE: This is technically not correct, since we might just wait as long
        // as we want before actually polling the future returned here
        // Nonetheless, we make this access here, since we do not want to keep
        // a reference to self in the future
        self.last_access = Instant::now();

        // NOTE: This is not written as a simple async function, because we don't want
        // to keep the mut self reference. This struct is used in a mutex, and we don't
        // want to hold the mutex across an await, but rather release it and then await
        // the future
        let req = self.sender.send_request(request).then(|response| {
            // TODO: Implement error handling here
            let Ok(response) = response else { todo!() };

            let status = response.status().as_u16();
            response.collect().map(move |response| {
                let Ok(response) = response else { todo!() };
                let response: Vec<u8> = response.to_bytes().into();

                Ok((status, response))
            })
        });

        Ok(req)
    }

    pub(crate) fn has_timed_out(&self) -> bool {
        Instant::now() - self.last_access > self.config.connection_timeout
    }
}
