use crate::config::OtlspClientConfig;
use futures::{FutureExt, TryFutureExt};
use http_body_util::BodyExt;
use hyper::{
    Request,
    client::conn::http1::SendRequest,
    header::{HOST, HeaderValue, USER_AGENT},
};
use otlsp_client::{OtlspClientBuilder, OtlspError};
use std::sync::Arc;
use url::Url;
use web_time::Instant;

#[derive(Debug)]
pub(crate) struct OtlspConnection {
    last_access: Instant,
    config: Arc<OtlspClientConfig>,
    host: String,
    sender: SendRequest<String>,
}

impl OtlspConnection {
    pub(crate) async fn new(config: Arc<OtlspClientConfig>, url: Url) -> Result<Self, OtlspError> {
        let proxy_url = config.proxy_url.as_ref().expect("Proxy url unset");

        let Some(host) = url.host_str() else {
            return Err(OtlspError::Unreachable("Cannot-be-a-base url".to_string()));
        };

        tracing::trace!("Creating otlsp connection to {} via {}", url, proxy_url);

        let sender = OtlspClientBuilder::new(proxy_url.clone())
            .with_webpki_roots()
            .handshake(url.clone())
            .await?;

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
    ) -> Result<impl Future<Output = Result<(u16, Vec<u8>), OtlspError>> + use<>, OtlspError> {
        assert_eq!(Some(self.host.as_str()), url.host_str());

        let mut url = url.clone();

        for (key, value) in params {
            url.query_pairs_mut().append_pair(key, value);
        }

        let request = Request::builder()
            .uri(url.as_str())
            .method("GET")
            // Add headers for host, agent and params
            .header(
                HOST,
                HeaderValue::from_str(&self.host).expect("Invalid host string"),
            )
            .header(
                USER_AGENT,
                HeaderValue::from_str(&self.config.agent).expect("Invalid user agent string "),
            )
            .body("".to_string())?;

        // NOTE: This is technically not correct, since we might just wait as long
        // as we want before actually polling the future returned here
        // Nonetheless, we make this access here, since we do not want to keep
        // a reference to self in the future
        self.last_access = Instant::now();

        // NOTE: This is not written as a simple async function, because we don't want
        // to keep the mut self reference. This struct is used in a mutex, and we don't
        // want to hold the mutex across an await, but rather release it and then await
        // the future
        let request = self
            .sender
            .send_request(request)
            .map_err(OtlspError::from)
            .and_then(|response| {
                let status = response.status().as_u16();
                response.collect().map(move |response| {
                    let response: Vec<u8> = response?.to_bytes().into();

                    tracing::debug!(
                        "Received {} bytes from request (status: {})",
                        response.len(),
                        status
                    );

                    Ok((status, response))
                })
            });

        Ok(request)
    }

    pub(crate) fn has_timed_out(&self) -> bool {
        Instant::now() - self.last_access > self.config.connection_timeout
    }
}
