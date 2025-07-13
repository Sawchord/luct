use luct_core::{CtLog, CtLogConfig, signature::SignatureValidationError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

mod request;
mod util;

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
    ) -> impl Future<Output = Result<String, ClientError>>;

    // TODO: Post
}

// pub struct DynClient(Arc<dyn Client>);

// impl Client for DynClient {
//     async fn get(&self, url: &Url, params: &(&str, &str)) -> Result<String, ClientError> {
//         self.0.get(url, params)
//     }
// }

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

    #[error("Signature validation of {0} against the logs key failed: {1}")]
    SignatureValidationFailed(&'static str, SignatureValidationError),

    #[error("Failed to validate a consistency path")]
    ConsistencyProofError,
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
