use luct_client::ClientError;
use luct_core::{CertificateError, tiling::TilingError};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ScannerError {
    #[error("Invalid certificate: {0}")]
    CertificateError(#[from] CertificateError),

    #[error("HTTP client error: {0}")]
    ClientError(#[from] ClientError),

    #[error("Failed to construct proof from tiles {0}")]
    TilingError(#[from] TilingError),
}
