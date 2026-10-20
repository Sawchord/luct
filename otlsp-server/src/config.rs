#[derive(Debug, Clone)]
pub struct OtlspConfig {
    pub buffer_size: usize,
    pub connection_timeout_seconds: u64,
}
