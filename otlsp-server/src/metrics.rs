use prometheus::{Registry, default_registry};

#[derive(Debug, Clone)]
pub struct OtlspMetrics {}

impl OtlspMetrics {
    pub fn new_with_registry(registry: &Registry) -> Self {
        todo!()
    }

    pub fn new() -> Self {
        Self::new_with_registry(default_registry())
    }
}
