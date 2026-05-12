use hyper::{Error as HyperError, http::Error as HttpError};
use rustls::{Error as RustlsError, server::VerifierBuilderError};
use std::{io, sync::Arc};
use thiserror::Error;

/// Errors that may occur while establishing and maintaining an oblivious TLS proxy connections
#[derive(Debug, Clone, Error)]
pub enum OtlspError {
    /// The proxy was unreachable
    #[error("Network unreachable: {0}")]
    Unreachable(String),

    /// The proxy was unreachable
    #[error("Network unreachable: {0}")]
    UnreachableStd(Arc<dyn std::error::Error + Send + Sync>),

    /// An error occured within the TCP connection between proxy and destination server
    #[error("TCP stream error: {0}")]
    Tcp(Arc<io::Error>),

    /// An error occured within the TLS connection between client and destination server
    #[error("TLS error: {0}")]
    Tls(#[from] RustlsError),

    /// And error occured while parsing HTTP artifacts
    #[error("HTTP error: {0}")]
    Http(Arc<HyperError>),

    /// And error occured while parsing HTTP artifacts
    #[error("HTTP error: {0}")]
    HttpBody(Arc<HttpError>),

    /// And error occured while loading the trust stire
    #[error("Error building certificate verifier: {0}")]
    VerifierBuilderError(#[from] VerifierBuilderError),

    /// Failed to parse the destination server urls
    #[error("Invalid dns name")]
    InvalidDnsNameError,

    /// Unknown error
    #[error("Unknown error")]
    Unknown,
}

impl From<HyperError> for OtlspError {
    fn from(error: HyperError) -> Self {
        Self::Http(Arc::new(error))
    }
}

impl From<HttpError> for OtlspError {
    fn from(error: HttpError) -> Self {
        Self::HttpBody(Arc::new(error))
    }
}

impl From<io::Error> for OtlspError {
    fn from(error: io::Error) -> Self {
        Self::Tcp(Arc::new(error))
    }
}
