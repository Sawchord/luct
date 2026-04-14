mod browser;
mod client;
mod error;

pub use client::OtlspClientBuilder;
pub use error::OtlspError;

// TODO: Native implementation using a native websocket client
