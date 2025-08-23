use crate::error::OtlspError;
use rustls::{ClientConfig, ClientConnection, RootCertStore, client::WebPkiServerVerifier};
use rustls_pki_types::ServerName;
use std::sync::Arc;
use url::Url;

pub struct Client {}

impl Client {
    pub async fn new(proxy: Url) -> Result<Self, OtlspError> {
        let server_name = ServerName::try_from(proxy.domain().unwrap())
            .unwrap()
            .to_owned();

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

        let mut conn = ClientConnection::new(Arc::new(config), server_name).unwrap();

        todo!()
    }
}
