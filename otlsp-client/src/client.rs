use crate::{browser::async_stream::AsyncStream, error::OtlspError};
use hyper::{body::Body, client::conn::http1::SendRequest};
use rustls::{ClientConfig, ClientConnection, RootCertStore, client::WebPkiServerVerifier};
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

        let stream = AsyncStream::new(conn, self.proxy, dst).await?;
        let (sender, connection) = hyper::client::conn::http1::handshake::<_, B>(stream).await?;

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
    use hyper::{Request, body::Buf, header::HOST};
    use std::io::ErrorKind;
    use tracing::Level;
    use tracing_subscriber::{Registry, layer::SubscriberExt};
    use tracing_wasm::{ConsoleConfig, WASMLayer, WASMLayerConfigBuilder};
    use wasm_bindgen_test::wasm_bindgen_test;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn smoke_test() {
        tracing();

        let (status, response) =
            get_request("https://tuscolo2026h2.skylight.geomys.org", "/checkpoint")
                .await
                .unwrap();

        tracing::info!("Status: {}", status);
        tracing::info!("{}", String::from_utf8_lossy(&response));
    }

    #[wasm_bindgen_test]
    async fn permission_denied_test() {
        tracing();

        // This url is not a log and therefore will not be enabled in on a proxy
        let result = get_request("https://google.com", "/").await;
        assert_error(result, ErrorKind::PermissionDenied);
    }

    // #[wasm_bindgen_test]
    // async fn invalid_url_test() {
    //     tracing();

    //     // This url is not fully qualified and should therefore be rejected
    //     let result = get_request("google.com", "/").await;
    //     assert_error(result, ErrorKind::InvalidInput);
    // }

    async fn get_request(url: &str, path: &str) -> Result<(u16, Vec<u8>), OtlspError> {
        let url = Url::parse(url).unwrap();
        let host = url.host_str().unwrap().to_string();

        let mut sender =
            OtlspClientBuilder::new(Url::parse("https://node.luct.dev/otlsp").unwrap())
                .with_webpki_roots()
                //.with_root_cert(Certificate::from_pem(include_str!("../e2e-test/ca.crt")).unwrap())
                .handshake(url)
                .await?;

        let req = Request::builder()
            .uri(path)
            .header(HOST, host)
            .method("GET")
            .body("".to_string())?;

        let res = sender.send_request(req).await?;

        let status = res.status().as_u16();
        let mut response = res.collect().await?.to_bytes();
        let response = response.copy_to_bytes(response.remaining()).to_vec();

        Ok((status, response))
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

    fn assert_error(result: Result<(u16, Vec<u8>), OtlspError>, error: ErrorKind) {
        match result {
            Err(OtlspError::Tcp(err)) => {
                let kind = err.kind();
                assert_eq!(kind, error);
            }
            a => panic!("Unexpected result {:?}", a),
        };
    }
}
