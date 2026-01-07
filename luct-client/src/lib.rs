use luct_core::{
    CertificateError, CheckSeverity, CtLog, CtLogConfig, Severity, SignatureValidationError,
    v1::SignedTreeHead,
};
use thiserror::Error;
use url::Url;

mod request;
#[cfg(feature = "reqwest")]
pub mod reqwest;
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

pub trait Client {
    fn get(
        &self,
        url: &Url,
        params: &[(&str, &str)],
    ) -> impl Future<Output = Result<(u16, String), ClientError>>;

    // TODO(Submission support): Post calls for submission support
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

    #[error("Request to {url} returned error: {code}: {msg}")]
    ResponseError { url: String, code: u16, msg: String },
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
            ClientError::JsonError { .. } => Severity::Unsafe,
            ClientError::CertificateError(err) => err.severity(),
            ClientError::SignatureValidationFailed(_, err) => err.severity(),
            ClientError::ConsistencyProofError => Severity::Unsafe,
            ClientError::AuditProofError => Severity::Unsafe,
            ClientError::ConnectionError(_) => Severity::Inconclusive,
            ClientError::ResponseError { .. } => Severity::Inconclusive,
        }
    }
}

impl<C: Client> CtClient<C> {
    pub async fn update_sth_v1(
        &self,
        old_sth: Option<&SignedTreeHead>,
    ) -> Result<SignedTreeHead, ClientError> {
        let new_sth = self.get_sth_v1().await?;

        // If we have no old seth, simply return the new one
        let Some(old_sth) = old_sth else {
            return Ok(new_sth);
        };

        if old_sth == &new_sth {
            return Ok(new_sth);
        }

        self.check_consistency_v1(old_sth, &new_sth).await?;
        Ok(new_sth)
    }
}

#[cfg(all(test, feature = "reqwest"))]
mod tests {
    use super::*;
    use crate::reqwest::ReqwestClient;
    use luct_core::{
        CertificateChain, CtLogConfig,
        v1::{SignedTreeHead, responses::GetSthResponse},
    };

    const ARGON2025H2: &str = "
        version = 1
        url = \"https://ct.googleapis.com/logs/us1/argon2025h2/\"
        key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEr+TzlCzfpie1/rJhgxnIITojqKk9VK+8MZoc08HjtsLzD8e5yjsdeWVhIiWCVk6Y6KomKTYeKGBv6xVu93zQug==\"
        mmd = 86400
    ";

    const ARGON2025H2_STH_0506: &str = "{
        \"tree_size\":1329315675,
        \"timestamp\":1751738269891,
        \"sha256_root_hash\":\"NEFqldTJt2+wE/aaaQuXeADdWVV8IGbwhLublI7QaMY=\",
        \"tree_head_signature\":\"BAMARjBEAiA9rna9/avaKTald7hHrldq8FfB4FDAaNyB44pplv71agIgeD0jj2AhLnvlaWavfFZ3BdUglauz36rFpGLYuLBs/O8=\"
    }";
    const CERT_CHAIN_GOOGLE_COM: &str = include_str!("../../testdata/google-chain.pem");

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn sth_consistency() {
        let client = get_client();

        let old_sth: GetSthResponse = serde_json::from_str(ARGON2025H2_STH_0506).unwrap();
        let old_sth = SignedTreeHead::from(old_sth);

        client.update_sth_v1(Some(&old_sth)).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn sct_inclusion() {
        let client = get_client();

        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        cert.verify_chain().unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let sth = client.get_sth_v1().await.unwrap();
        client
            .check_embedded_sct_inclusion_v1(&scts[0], &sth, &cert)
            .await
            .unwrap();
    }

    fn get_client() -> CtClient<ReqwestClient> {
        let config: CtLogConfig = toml::from_str(ARGON2025H2).unwrap();
        let client = ReqwestClient::new();
        CtClient::new(config, client)
    }

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn get_roots() {
        let client = get_client();

        let roots = client.get_roots_v1().await.unwrap();
        assert!(!roots.is_empty())
    }
}
