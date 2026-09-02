use crate::{OtlspConfig, OtlspMetrics};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct OtlspState {
    pub config: Arc<OtlspConfig>,
    pub metrics: OtlspMetrics,
}
