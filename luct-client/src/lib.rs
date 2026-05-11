#![forbid(unsafe_code)]

use luct_core::{
    CtLog, CtLogConfig, SignatureValidationError,
    tiling::{ParseCheckpointError, TilingError},
    tree::ProofValidationError,
};
use std::{error::Error, fmt::Debug, sync::Arc};
use thiserror::Error;
use url::Url;

pub use impls::*;

mod impls;
mod request;
mod util;

// TODO: Fetch entries API
// TODO: Tests with a mock client

/// Wrapper around [`Client`], that implements fetching and validation logic
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtClient<C> {
    log: CtLog,
    client: C,
}

impl<C> CtClient<C> {
    pub fn new(config: CtLogConfig, client: C) -> Self {
        Self {
            log: CtLog::new(config),
            client,
        }
    }

    pub fn log(&self) -> &CtLog {
        &self.log
    }
}

/// Backend client implementation trait
///
/// This trait needs to be implemented by clients to be used by auditors, monitors etc.
pub trait Client: Debug {
    /// Make a GET request to fetch some [`String`] data
    ///
    /// # Arguments
    /// - `url`: the [`Url`] to connect to
    /// - `params`: Key-value pairs of query parameters to be included in the request
    ///
    /// # Returns
    /// - **On success**:
    ///     - The HTTP status code
    ///     - The data as a [`String`]
    /// - **On failure**: The [`ClientError`] describing what went wrong
    fn get(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> impl Future<Output = Result<(u16, Arc<String>), ClientError>>;

    /// Make a GET request to fetch some binary data
    ///
    /// # Arguments
    /// - `url`: the [`Url`] to connect to
    /// - `params`: Key-value pairs of query parameters to be included in the request
    ///
    /// # Returns
    /// - **On success**:
    ///     - The HTTP status code
    ///     - The data as a [`Vec<u8>`]
    /// - **On failure**: The [`ClientError`] describing what went wrong
    fn get_bin(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> impl Future<Output = Result<(u16, Arc<Vec<u8>>), ClientError>>;

    // TODO(Submission support): Post calls for submission support
}

/// Error returned by [`Client`] implementation when a request fails
#[derive(Debug, Clone, Error)]
pub enum ClientError {
    /// A client attempted to make a request, that is not supported by a log with this version
    #[error("The version of the log is not supported by this client")]
    UnsupportedVersion,

    /// A request was returned but the client failed to parse the JSON in the resonse
    #[error("Failed to parse JSON: line: {line}, column: {column}")]
    JsonError { line: usize, column: usize },

    /// Verification of a signature failed
    #[error("Signature validation of {0} against the logs key failed: {1}")]
    SignatureValidationFailed(&'static str, SignatureValidationError),

    /// Validating a consistency proof failed
    #[error("Failed to validate a consistency path: {0}")]
    ConsistencyProofError(ProofValidationError),

    /// Validating an audit proof failed
    #[error("Failed to validate an audit path: {0}")]
    AuditProofError(ProofValidationError),

    // TODO: Remove
    /// The connection failed
    #[error("Failed to connect to host: {0}")]
    ConnectionError(String),

    /// The connection failed
    #[error("Failed to connect to host: {0}")]
    ConnectionErrorStd(Arc<dyn Error + Send + Sync>),

    /// The request failed, the server returned a response code other than 200
    #[error("Request to {url} returned error: {code}: {msg}")]
    ResponseError { url: String, code: u16, msg: String },

    /// Failed to parse a checkpoint note
    #[error("Failed parsing checkpoint: {0}")]
    Checkpoint(#[from] ParseCheckpointError),

    /// An error specific to the tiling API occured
    #[error("Tiling error: {0}")]
    TilingError(#[from] TilingError),

    // TODO: Remove
    /// The STH could not be parsed
    #[error("The STH could not be parsed")]
    SthError,
}

impl From<serde_json::Error> for ClientError {
    fn from(value: serde_json::Error) -> Self {
        ClientError::JsonError {
            line: value.line(),
            column: value.column(),
        }
    }
}
