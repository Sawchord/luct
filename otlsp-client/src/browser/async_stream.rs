use crate::{browser::ws_stream::WsStream, console_log};
use futures::io;
use hyper::rt::ReadBufCursor;
use rustls::{ClientConnection, StreamOwned};
use std::{
    cell::RefCell,
    io::{Read, Write},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

#[derive(Debug)]
pub(crate) struct AsyncStream {
    pub(crate) stream: StreamOwned<ClientConnection, WsStream>,
    pub(crate) waker: Rc<RefCell<Vec<Waker>>>,
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
