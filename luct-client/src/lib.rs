use luct_core::{CertificateError, CtLog, CtLogConfig, signature::SignatureValidationError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

mod request;
#[cfg(feature = "reqwest")]
pub mod reqwest;
mod util;

// TODO: Fetch entries API
// TODO: Update STH API
// TODO: Tests with a mock client

pub struct CtClient<C> {
    config: CtClientConfig,
    log: CtLog,
    client: C,
}

pub trait Client {
    fn get(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> impl Future<Output = Result<(u16, String), ClientError>>;

    // TODO: Post calls for submission support
}

impl<C> CtClient<C> {
    pub fn new(config: CtClientConfig, client: C) -> Self {
        Self {
            log: CtLog::new(config.log.clone()),
            config,
            client,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClientError {
    #[error("The version of the log is not supported by this client")]
    UnsupportedVersion,

    #[error("Failed to parse JSON: line: {line}, column: {column}")]
    JsonError { line: usize, column: usize },

    #[error("Invalid certificate: {0}")]
    CertificateError(#[from] CertificateError),

    #[error("Signature validation of {0} against the logs key failed: {1}")]
    SignatureValidationFailed(&'static str, SignatureValidationError),

    #[error("Failed to validate a consistency path")]
    ConsistencyProofError,

    #[error("Failed to validate an audit path")]
    AuditProofError,

    #[error("Failed to connect to host: {0}")]
    ConnectionError(String),

    #[error("The server returned error: {code}: {msg}")]
    ResponseError { code: u16, msg: String },
}

impl From<serde_json::Error> for ClientError {
    fn from(value: serde_json::Error) -> Self {
        ClientError::JsonError {
            line: value.line(),
            column: value.column(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtClientConfig {
    /// The configuration of the log itself
    log: CtLogConfig,

    /// Fetch the values from another url instead
    fetch_url: Option<Url>,
}
