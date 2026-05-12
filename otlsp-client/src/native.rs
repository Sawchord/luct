use crate::{OtlspError, WebsocketStream, async_stream::WsAsyncStream};
use hyper::{body::Body, client::conn::http1::Connection};
use std::{sync::Arc, task::Context};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use url::Url;
#[derive(Debug)]
pub struct NativeWebsocketStream(WebSocketStream<MaybeTlsStream<TcpStream>>);

impl WebsocketStream for NativeWebsocketStream {
    async fn new(proxy: Url, mut dst: Url) -> Result<Self, OtlspError> {
        dst.set_path("");
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

    fn spawn<B>(connection: Connection<WsAsyncStream<Self>, B>)
    where
        B: Body,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
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
