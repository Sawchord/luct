use crate::{AsyncStream, OtlspError, WebsocketStream};
use hyper::{body::Body, client::conn::http1::Connection, rt};
use rustls::{ClientConnection, StreamOwned};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use url::Url;

#[derive(Debug)]
pub struct NativeAsyncStream(StreamOwned<ClientConnection, NativeWebsocketStream>);

impl AsyncStream for NativeAsyncStream {
    async fn create(
        conn: rustls::ClientConnection,
        proxy: url::Url,
        dst: url::Url,
    ) -> Result<Self, OtlspError> {
        // Setup the underlying websocket stream
        let ws_stream = NativeWebsocketStream::new(proxy, dst).await?;

        // Initiate the connection
        let stream = StreamOwned::new(conn, ws_stream);
        Ok(Self(stream))
    }

    fn spawn<B>(_connection: Connection<Self, B>)
    where
        B: Body,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        todo!()
    }
}

impl rt::Read for NativeAsyncStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: rt::ReadBufCursor<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        todo!()
    }
}

impl rt::Write for NativeAsyncStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        todo!()
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        todo!()
    }
}

#[derive(Debug)]
pub struct NativeWebsocketStream(WebSocketStream<MaybeTlsStream<TcpStream>>);

impl WebsocketStream for NativeWebsocketStream {
    async fn new(proxy: Url, mut dst: Url) -> Result<Self, OtlspError> {
        let request_string = format!("{}?to={}", proxy.as_str(), dst.as_str());

        let (stream, _response) = connect_async(&request_string)
            .await
            .map_err(|err| OtlspError::UnreachableStd(Arc::new(err)))?;
        Ok(Self(stream))
    }

    fn close(&self) -> std::io::Result<()> {
        todo!()
    }

    fn enqueue_waker(&self, cx: &Context<'_>) {
        todo!()
    }
}

impl std::io::Read for NativeWebsocketStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

impl std::io::Write for NativeWebsocketStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!()
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}
