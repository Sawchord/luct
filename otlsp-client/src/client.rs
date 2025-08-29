use crate::{error::OtlspError, ws_stream::WsStream};
use hyper::rt::{Read, ReadBufCursor, Write};
use rustls::{ClientConfig, ClientConnection, RootCertStore, Stream, client::WebPkiServerVerifier};
use rustls_pki_types::ServerName;
use std::{
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
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

        let server_name = ServerName::try_from(dst.host_str().unwrap())
            .unwrap()
            .to_owned();
        let mut conn = ClientConnection::new(Arc::new(config), server_name).unwrap();

        let mut ws_stream = WsStream::new(proxy, dst).await?;

        // Initiate the connection
        let waker = ws_stream.waker();
        let tls = Stream::new(&mut conn, &mut ws_stream);
        let (sender, connection) =
            hyper::client::conn::http1::handshake::<_, String>(AsyncStream { stream: tls, waker })
                .await
                .unwrap();

        // TODO: Send connection to the web-sys executor

        todo!()
    }
}

#[derive(Debug)]
struct AsyncStream<'a> {
    stream: Stream<'a, ClientConnection, WsStream>,
    waker: Arc<Mutex<Vec<Waker>>>,
}

impl<'a> Read for AsyncStream<'a> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: ReadBufCursor<'_>,
    ) -> Poll<Result<(), io::Error>> {
        todo!()
    }
}

impl<'a> Write for AsyncStream<'a> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn set_test() {
        let client = Client::new(
            Url::parse("ws://127.0.0.1:3000").unwrap(),
            Url::parse("https://127.0.0.1:8080").unwrap(),
        )
        .await
        .unwrap();
        todo!()
    }
}
