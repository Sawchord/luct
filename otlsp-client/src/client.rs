use crate::{error::OtlspError, ws_stream::WsStream};
use rustls::{ClientConfig, ClientConnection, RootCertStore, Stream, client::WebPkiServerVerifier};
use rustls_pki_types::ServerName;
use std::sync::Arc;
use url::Url;

pub struct Client {}

impl Client {
    pub async fn new(proxy: Url, dst: Url) -> Result<Self, OtlspError> {
        // Set up client config, using rustcrypto as webpki roots (again with rustcrypto)
        let config = ClientConfig::builder_with_provider(rustls_rustcrypto::provider().into())
            .with_protocol_versions(&[&rustls::version::TLS13])
            .unwrap()
            .with_webpki_verifier(
                WebPkiServerVerifier::builder_with_provider(
                    RootCertStore {
                        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
                    }
                    .into(),
                    rustls_rustcrypto::provider().into(),
                )
                .build()
                .unwrap(),
            )
            .with_no_client_auth();

        let server_name = ServerName::try_from(dst.domain().unwrap())
            .unwrap()
            .to_owned();
        let mut conn = ClientConnection::new(Arc::new(config), server_name).unwrap();

        // TODO: Setup the websocket stream and connect it to rustls
        let mut ws_stream = WsStream::new(proxy).await?;
        let mut tls = Stream::new(&mut conn, &mut ws_stream);

        // TODO: Initiate the connection
        //let (sender, connection) = hyper::client::conn::http1::handshake(tls).await.unwrap();

        // TODO: Send connection to the web-sys executor

        todo!()
    }
}
