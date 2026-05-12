use crate::{AsyncStream, OtlspError};
use hyper::{body::Body, client::conn::http1::Connection, rt};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug)]
pub struct NativeAsyncStream {}

impl AsyncStream for NativeAsyncStream {
    async fn create(
        conn: rustls::ClientConnection,
        proxy: url::Url,
        dst: url::Url,
    ) -> Result<Self, OtlspError> {
        todo!()
    }

    fn spawn<B>(connection: Connection<Self, B>)
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
