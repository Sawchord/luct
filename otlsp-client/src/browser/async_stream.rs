use crate::{OtlspError, browser::ws_stream::WsStream};
use futures::io;
use hyper::rt::ReadBufCursor;
use rustls::{ClientConnection, StreamOwned};
use std::{
    io::{ErrorKind, Read, Write},
    pin::Pin,
    task::{Context, Poll},
};
use url::Url;

#[derive(Debug)]
pub(crate) struct AsyncStream(StreamOwned<ClientConnection, WsStream>);

impl AsyncStream {
    pub(crate) async fn new(
        conn: ClientConnection,
        proxy: Url,
        dst: Url,
    ) -> Result<Self, OtlspError> {
        // Setup the underlying websocket stream
        let ws_stream = WsStream::new(proxy, dst).await?;

        // Initiate the connection
        let stream = StreamOwned::new(conn, ws_stream);
        Ok(Self(stream))
    }
}

impl hyper::rt::Read for AsyncStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut read_buf: ReadBufCursor<'_>,
    ) -> Poll<Result<(), io::Error>> {
        let mut buf = [0u8; 1500];

        // Try to read the inner stream
        match self.0.read(&mut buf) {
            // If we got data back, we return it
            Ok(read) => {
                // TODO: Handle situation where read+buf has not enough space
                tracing::trace!("async stream read {} bytes", read);
                read_buf.put_slice(&buf[..read]);
                Poll::Ready(Ok(()))
            }
            // If we get an Interrupted error, we add the waker to waker,
            // such that the task gets woken up if the WsStream receives new bytes
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                tracing::trace!("async read: stream would block");
                self.0.sock.enqueue_waker(cx);
                Poll::Pending
            }
            // Other errors are being returned verbatim
            Err(err) => {
                tracing::trace!("Error reading async stream: {:?}", err);
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
        match self.0.write_all(buf) {
            Ok(()) => {
                tracing::trace!("async stream wrote {} bytes", buf.len());
                Poll::Ready(Ok(buf.len()))
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                tracing::trace!("async write: stream would block");
                self.0.sock.enqueue_waker(cx);
                Poll::Pending
            }
            Err(err) => {
                tracing::trace!("Error writing async stream: {:?}", err);
                Err(err)?
            }
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        // TODO: Implement
        tracing::warn!("Called poll_flush which is not implemented");
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.0.sock.close() {
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                tracing::debug!("poll_shutdown: waiting on close");
                self.0.sock.enqueue_waker(cx);
                Poll::Pending
            }
            result => {
                tracing::debug!("poll_shutdown: shutting down with status {:?}", result);
                Poll::Ready(result)
            }
        }
    }
}
