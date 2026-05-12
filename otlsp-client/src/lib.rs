//! Implementation of the client side of oblivious TLS proxy
//!
//! In the browser, it uses the browser native websocket API

#![forbid(unsafe_code)]

mod browser;
mod client;
mod error;

pub use browser::async_stream::WsAsyncStream;
pub use client::OtlspConnectionBuilder;
pub use error::OtlspError;
use hyper::{body::Body, client::conn::http1::Connection, rt};
use rustls::ClientConnection;
use std::future::Future;
use url::Url;

pub trait AsyncStream: rt::Write + rt::Read + Sized {
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

// TODO: Native implementation using a native websocket client
