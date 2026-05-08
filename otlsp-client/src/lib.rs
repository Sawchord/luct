//! Implementation of the client side of oblivious TLS proxy
//!
//! In the browser, it uses the browser native websocket API

#![forbid(unsafe_code)]

mod browser;
mod client;
mod error;

pub use client::OtlspClientBuilder;
pub use error::OtlspError;

// TODO: Native implementation using a native websocket client
