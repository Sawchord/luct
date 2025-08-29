use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OtlspError {
    #[error("Network unreachable: {0}")]
    Unreachable(String),
}
