//! Implementation of the client side of oblivious TLS proxy
//!
//! In the browser, it uses the browser native websocket API

#![forbid(unsafe_code)]

mod async_stream;
mod browser;
mod client;
mod error;
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
mod native;

pub use client::OtlspConnectionBuilder;
pub use error::OtlspError;
use hyper::{body::Body, client::conn::http1::Connection, rt};
use rustls::ClientConnection;
use std::{future::Future, io, task::Context};
use url::Url;

#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
pub use browser::BrowserWebsocketStream;
#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
pub type DefaultWebsocketStream = BrowserWebsocketStream;

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub use native::NativeWebsocketStream;

#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
pub type DefaultWebsocketStream = NativeWebsocketStream;

pub trait AsyncStream: rt::Write + rt::Read + Sized + Unpin {
    fn create(
        conn: ClientConnection,
        proxy: Url,
        dst: Url,
    ) -> impl Future<Output = Result<Self, OtlspError>>;

    fn spawn<B>(connection: Connection<Self, B>)
    where
        B: Body,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;
}

pub trait WebsocketStream: io::Read + io::Write + Sized + Unpin + 'static {
    fn new(proxy: Url, dst: Url) -> impl Future<Output = Result<Self, OtlspError>>;
    fn close(&self) -> io::Result<()>;
    fn enqueue_waker(&self, cx: &Context<'_>);

    fn spawn<B>(connection: Connection<async_stream::WsAsyncStream<Self>, B>)
    where
        B: Body + Send,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>;
}

// TODO: Native implementation using a native websocket client
