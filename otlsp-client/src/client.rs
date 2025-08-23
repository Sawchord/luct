use crate::error::OtlspError;
use http_body_util::Empty;
use hyper::body::Bytes;
//use hyper_rustls::{ConfigBuilderExt, HttpsConnectorBuilder};
//use hyper_util::client::legacy::Client as HyperClient;
use rustls::ClientConfig;
use std::sync::Arc;
use url::Url;

pub struct Client {
    tls_config: Arc<ClientConfig>,
}

impl Client {
    pub fn new(proxy: Url) -> Result<Self, OtlspError> {
        // let https = HttpsConnectorBuilder::new()
        //     .with_provider_and_webpki_roots(Arc::new(rustls_rustcrypto::provider()))
        //     .unwrap()
        //     .https_or_http()
        //     .enable_all_versions()
        //     .build();

        // let config = ClientConfig::builder_with_provider(rustls_rustcrypto::provider().into())
        //     .with_safe_default_protocol_versions()
        //     .unwrap();
        // .with_webpki_roots()
        // .with_no_client_auth();

        // let client: HyperClient<_, Empty<Bytes>> =
        //     HyperClient::builder(futures::ready!(())).build(https);
        todo!()
    }
}
