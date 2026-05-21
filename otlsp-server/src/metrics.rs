use prometheus::{
    CounterVec, Opts, Registry, default_registry, register_counter_vec_with_registry,
};

#[derive(Debug, Clone)]
pub struct OtlspMetrics {
    pub(crate) connections_opened: CounterVec,
    pub(crate) connections_closed: CounterVec,
    pub(crate) connection_errors: CounterVec,

    pub(crate) bytes_tx: CounterVec,
    pub(crate) bytes_rx: CounterVec,
}

impl OtlspMetrics {
    pub fn new_with_registry(registry: &Registry) -> Self {
        Self {
            connections_opened: register_counter_vec_with_registry!(
                Opts::new(
                    "connections_opened",
                    "Number of connections that where succesfully opened"
                ),
                &["destination"],
                registry
            )
            .unwrap(),
            connections_closed: register_counter_vec_with_registry!(
                Opts::new(
                    "connections_closed",
                    "Number of connections that where closed, possibly with an error"
                ),
                &["destination", "client_init", "error"],
                registry
            )
            .unwrap(),
            connection_errors: register_counter_vec_with_registry!(
                Opts::new(
                    "connection_errors",
                    "Count the number of connections that where closed"
                ),
                &["destination", "error"],
                registry
            )
            .unwrap(),
            bytes_tx: register_counter_vec_with_registry!(
                Opts::new("bytes_tx", "Number of bytes sent from client to server"),
                &["destination"],
                registry
            )
            .unwrap(),
            bytes_rx: register_counter_vec_with_registry!(
                Opts::new("bytes_rx", "Number of bytes sent form server to client"),
                &["destination"],
                registry
            )
            .unwrap(),
        }
    }
}

impl Default for OtlspMetrics {
    fn default() -> Self {
        Self::new_with_registry(default_registry())
    }
}
