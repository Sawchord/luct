use crate::{console_log, error::OtlspError, ws_stream::WsStream};
use hyper::{client::conn::http1::SendRequest, rt::ReadBufCursor};
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, StreamOwned, client::WebPkiServerVerifier,
};
use rustls_pki_types::ServerName;
use std::{
    io::{self, Read, Write},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use url::Url;

// TODO: Builder, to allow setting certs manually
// TODO: Check that starvations etc can't happen
// TODO: Replace unwraps with specific errors
// TODO: Try to get error response bodys

pub struct Client {}

impl Client {
    pub async fn create(proxy: Url, dst: Url) -> Result<SendRequest<String>, OtlspError> {
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
        let conn = ClientConnection::new(Arc::new(config), server_name).unwrap();

        // Setup the underlying websocket stream
        let ws_stream = WsStream::new(proxy, dst).await?;

        // Initiate the connection
        let waker = ws_stream.waker();
        let tls = StreamOwned::new(conn, ws_stream);
        let (sender, connection) =
            hyper::client::conn::http1::handshake::<_, String>(AsyncStream { stream: tls, waker })
                .await
                .unwrap();

        // Send connection to the web-sys executor
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(err) = connection.await {
                console_log!("Connection failed: {:?}", err)
            }
        });

        Ok(sender)
    }
}

#[derive(Debug)]
struct AsyncStream {
    stream: StreamOwned<ClientConnection, WsStream>,
    waker: Arc<Mutex<Vec<Waker>>>,
}

impl hyper::rt::Read for AsyncStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut read_buf: ReadBufCursor<'_>,
    ) -> Poll<Result<(), io::Error>> {
        console_log!("Calling async read");
        let mut buf = [0u8; 1500];

        // Try to read the inner stream
        match self.stream.read(&mut buf) {
            // If we got data back, we return it
            Ok(read) => {
                read_buf.put_slice(&buf[..read]);
                Poll::Ready(Ok(()))
            }
            // If we get an Interrupted error, we add the waker to waker,
            // such that the task gets woken up if the WsStream receives new bytes
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {
                self.waker.lock().unwrap().push(cx.waker().clone());
                Poll::Pending
            }
            // Other errors are being returned verbatim
            Err(err) => {
                console_log!("Error reading Async Stream: {:?}", err);
                Err(err)?
            }
        }
    }
}

impl hyper::rt::Write for AsyncStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        console_log!("Calling async write");
        self.stream.write_all(buf)?;

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;
    use hyper::Request;
    use wasm_bindgen_test::wasm_bindgen_test;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn set_test() {
        let mut sender = Client::create(
            Url::parse("ws://127.0.0.1:3000").unwrap(),
            Url::parse("https://127.0.0.1:8080").unwrap(),
        )
        .await
        .unwrap();

        console_log!("Still alive");

        let req = Request::builder()
            .uri("/test")
            .body("".to_string())
            .unwrap();

        console_log!("Still alive");
        let mut res = sender.send_request(req).await.unwrap();

        todo!()
    }
}
