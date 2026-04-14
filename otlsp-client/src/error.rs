use hyper::{Error as HyperError, http::Error as HttpError};
use rustls::{Error as RustlsError, server::VerifierBuilderError};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum OtlspError {
    #[error("Network unreachable: {0}")]
    Unreachable(String),

    #[error("TLS error: {0}")]
    Tls(#[from] RustlsError),

    #[error("HTTP error: {0}")]
    Http(Arc<HyperError>),

    #[error("HTTP error: {0}")]
    HttpBody(Arc<HttpError>),

    #[error("Error building certificate verifier: {0}")]
    VerifierBuilderError(#[from] VerifierBuilderError),

    #[error("Invalid dns name")]
    InvalidDnsNameError,
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
