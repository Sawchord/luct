use crate::{
    browser::{async_stream::AsyncStream, ws_stream::WsStream},
    error::OtlspError,
};
use hyper::{body::Body, client::conn::http1::SendRequest};
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, StreamOwned, client::WebPkiServerVerifier,
};
use rustls_pki_types::{ServerName, TrustAnchor};
use std::sync::Arc;
use url::Url;
use x509_cert::{Certificate, der::Encode};

pub struct OtlspClientBuilder {
    proxy: Url,
    roots: Vec<TrustAnchor<'static>>,
}

impl OtlspClientBuilder {
    pub fn new(proxy: Url) -> Self {
        Self {
            proxy,
            roots: vec![],
        }
    }

    pub fn with_webpki_roots(mut self) -> Self {
        self.roots.extend_from_slice(webpki_roots::TLS_SERVER_ROOTS);
        self
    }

    pub fn with_root_cert(mut self, cert: Certificate) -> Self {
        self.roots.push(TrustAnchor {
            subject: cert
                .tbs_certificate
                .subject
                .to_der()
                .expect("Failed to parse DER for tbs_certificate")
                .into(),
            subject_public_key_info: cert
                .tbs_certificate
                .subject_public_key_info
                .to_der()
                .expect("Failed to parse DER for subject_public_key_info")
                .into(),
            name_constraints: None,
        });
        self
    }

    pub async fn handshake<B>(self, dst: Url) -> Result<SendRequest<B>, OtlspError>
    where
        B: Body + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        // Set up client config, using rustcrypto as webpki roots (again with rustcrypto)
        let config = ClientConfig::builder_with_provider(rustls_rustcrypto::provider().into())
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_webpki_verifier(
                WebPkiServerVerifier::builder_with_provider(
                    RootCertStore { roots: self.roots }.into(),
                    rustls_rustcrypto::provider().into(),
                )
                .build()?,
            )
            .with_no_client_auth();

        let server_name =
            ServerName::try_from(dst.host_str().ok_or(OtlspError::InvalidDnsNameError)?)
                .map_err(|_| OtlspError::InvalidDnsNameError)?
                .to_owned();
        let conn = ClientConnection::new(Arc::new(config), server_name)?;

        // Setup the underlying websocket stream
        let ws_stream = WsStream::new(self.proxy, dst).await?;

        // Initiate the connection
        let waker = ws_stream.waker();
        let tls = StreamOwned::new(conn, ws_stream);
        let (sender, connection) =
            hyper::client::conn::http1::handshake::<_, B>(AsyncStream { stream: tls, waker })
                .await?;

        // Send connection to the web-sys executor
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(err) = connection.await {
                tracing::error!("Connection failed: {:?}", err)
            }
        });

        Ok(sender)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use hyper::{Request, body::Buf};
    use tracing::Level;
    use tracing_subscriber::{Registry, layer::SubscriberExt};
    use tracing_wasm::{ConsoleConfig, WASMLayer, WASMLayerConfigBuilder};
    use wasm_bindgen_test::wasm_bindgen_test;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // NOTE: This test requires setup that can be found in the e2e test directory
    #[wasm_bindgen_test]
    async fn smoke_test() {
        tracing();

        let mut sender =
            OtlspClientBuilder::new(Url::parse("https://node.luct.dev/otlsp").unwrap())
                .with_webpki_roots()
                //.with_root_cert(Certificate::from_pem(include_str!("../e2e-test/ca.crt")).unwrap())
                .handshake(Url::parse("https://tuscolo2026h2.skylight.geomys.org").unwrap())
                //.handshake(Url::parse("https://google.com").unwrap())
                .await
                .unwrap();

        tracing::info!("Still alive");

        // TODO: Set user agent, host
        let req = Request::builder()
            //.uri("/tile/0/000")
            .uri("/checkpoint")
            .method("GET")
            .body("".to_string())
            .unwrap();

        tracing::info!("Still alive");
        let res = sender.send_request(req).await.unwrap();

        //assert_eq!(res.status(), 200);
        let status = res.status();
        //let mut response = res.collect().await.unwrap().aggregate();
        //let response = response.copy_to_bytes(response.remaining()).to_vec();
        let mut response = res.collect().await.unwrap().to_bytes();
        let response = response.copy_to_bytes(response.remaining()).to_vec();

        //const TEXT: &str = include_str!("../e2e-test/data/test.txt");
        //assert_eq!(Bytes::from(TEXT), response);
        tracing::info!("Status: {}", status);
        tracing::info!("{}", String::from_utf8_lossy(&response));

        panic!();
    }

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
}
