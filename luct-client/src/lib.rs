use luct_core::{
    CertificateError, CheckSeverity, CtLog, CtLogConfig, Severity, SignatureValidationError,
    tiling::ParseCheckpointError,
};
use std::{fmt::Debug, sync::Arc};
use thiserror::Error;
use url::Url;

pub use impls::*;

mod impls;
mod request;
mod util;

// TODO: Fetch entries API
// TODO: Tests with a mock client

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

pub trait Client: Debug {
    fn get(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> impl Future<Output = Result<(u16, Arc<String>), ClientError>>;

    fn get_bin(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> impl Future<Output = Result<(u16, Arc<Vec<u8>>), ClientError>>;

    // TODO(Submission support): Post calls for submission support
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClientError {
    #[error("The version of the log is not supported by this client")]
    UnsupportedVersion,

    #[error("Can not fetch tiles from non tiling log")]
    NonTilingLog,

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

    #[error("Request to {url} returned error: {code}: {msg}")]
    ResponseError { url: String, code: u16, msg: String },

    #[error("Failed parsing checkpoint: {0}")]
    Checkpoint(#[from] ParseCheckpointError),

    #[error("The tile that was returned by the log is malformed")]
    MalformedTile,

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

impl CheckSeverity for ClientError {
    fn severity(&self) -> Severity {
        match self {
            ClientError::UnsupportedVersion => Severity::Inconclusive,
            ClientError::NonTilingLog => Severity::Inconclusive,
            ClientError::JsonError { .. } => Severity::Unsafe,
            ClientError::CertificateError(err) => err.severity(),
            ClientError::SignatureValidationFailed(_, err) => err.severity(),
            ClientError::ConsistencyProofError => Severity::Unsafe,
            ClientError::AuditProofError => Severity::Unsafe,
            ClientError::ConnectionError(_) => Severity::Inconclusive,
            ClientError::ResponseError { .. } => Severity::Inconclusive,
            ClientError::Checkpoint(_) => Severity::Unsafe,
            ClientError::MalformedTile => Severity::Unsafe,
            ClientError::SthError => Severity::Unsafe,
        }
    }
}
