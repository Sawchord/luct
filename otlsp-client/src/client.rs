use crate::{console_log, error::OtlspError, ws_stream::WsStream};
use hyper::{body::Body, client::conn::http1::SendRequest, rt::ReadBufCursor};
use rustls::{
    ClientConfig, ClientConnection, RootCertStore, StreamOwned, client::WebPkiServerVerifier,
};
use rustls_pki_types::{ServerName, TrustAnchor};
use std::{
    cell::RefCell,
    io::{self, Read, Write},
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Waker},
};
use url::Url;
use x509_cert::{Certificate, der::Encode};

// TODO: Builder, to allow setting certs manually
// TODO: Check that starvations etc can't happen
// TODO: Replace unwraps with specific errors
// TODO: Try to get error response bodys

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
            subject: cert.tbs_certificate.subject.to_der().unwrap().into(),
            subject_public_key_info: cert
                .tbs_certificate
                .subject_public_key_info
                .to_der()
                .unwrap()
                .into(),
            name_constraints: None,
        });
        self
    }

    // TODO: Remove unwraps and return OtlspError instead
    pub async fn handshake<B>(self, dst: Url) -> Result<SendRequest<B>, OtlspError>
    where
        B: Body + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        // Set up client config, using rustcrypto as webpki roots (again with rustcrypto)
        let config = ClientConfig::builder_with_provider(rustls_rustcrypto::provider().into())
            .with_protocol_versions(&[&rustls::version::TLS13])
            .unwrap()
            .with_webpki_verifier(
                WebPkiServerVerifier::builder_with_provider(
                    RootCertStore { roots: self.roots }.into(),
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
        let ws_stream = WsStream::new(self.proxy, dst).await?;

        // Initiate the connection
        let waker = ws_stream.waker();
        let tls = StreamOwned::new(conn, ws_stream);
        let (sender, connection) =
            hyper::client::conn::http1::handshake::<_, B>(AsyncStream { stream: tls, waker })
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
    waker: Rc<RefCell<Vec<Waker>>>,
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
                // TODO: Handle situation where read+buf has not enough space
                console_log!("AsyncStream read {} bytes", read);
                read_buf.put_slice(&buf[..read]);
                Poll::Ready(Ok(()))
            }
            // If we get an Interrupted error, we add the waker to waker,
            // such that the task gets woken up if the WsStream receives new bytes
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                console_log!("AsyncStream would block");
                self.waker.borrow_mut().push(cx.waker().clone());
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
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        console_log!("Calling async write with {} bytes", buf.len());

        match self.stream.write_all(buf) {
            Ok(()) => Poll::Ready(Ok(buf.len())),
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                console_log!("AsyncStream would block");
                self.waker.borrow_mut().push(cx.waker().clone());
                Poll::Pending
            }
            Err(err) => {
                console_log!("Error writing Async Stream: {:?}", err);
                Err(err)?
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        // TODO: Implement
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // #![allow(dead_code)]
    // use super::*;
    // use http_body_util::BodyExt;
    // use hyper::{
    //     Request,
    //     body::{Buf, Bytes},
    // };
    // use wasm_bindgen_test::wasm_bindgen_test;
    // use x509_cert::der::DecodePem;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // NOTE: This test requires setup that can be found in the e2e test directory
    // #[wasm_bindgen_test]
    // async fn e2e_test() {
    //     let mut sender = OtlspClientBuilder::new(Url::parse("ws://127.0.0.1:3000").unwrap())
    //         .with_webpki_roots()
    //         .with_root_cert(Certificate::from_pem(include_str!("../e2e-test/ca.crt")).unwrap())
    //         .handshake(
    //             Url::parse("https://localhost:8080").unwrap(),
    //             //Url::parse("https://google.com:443").unwrap(),
    //         )
    //         .await
    //         .unwrap();

    //     console_log!("Still alive");

    //     let req = Request::builder()
    //         .uri("/")
    //         .method("GET")
    //         .body("".to_string())
    //         .unwrap();

    //     console_log!("Still alive");
    //     let res = sender.send_request(req).await.unwrap();

    //     assert_eq!(res.status(), 200);
    //     let mut response = res.collect().await.unwrap().aggregate();
    //     let response = response.copy_to_bytes(response.remaining()).to_vec();

    //     const TEXT: &str = include_str!("../e2e-test/data/test.txt");
    //     assert_eq!(Bytes::from(TEXT), response);
    //     console_log!("{}", String::from_utf8_lossy(&response));
    // }
}
