use prometheus::{
    IntCounterVec, Opts, Registry, default_registry, register_int_counter_vec_with_registry,
};
use std::io::ErrorKind;

#[derive(Debug, Clone)]
pub struct OtlspMetrics {
    connections_opened: IntCounterVec,
    connections_closed: IntCounterVec,
    connection_errors: IntCounterVec,

    bytes_tx: IntCounterVec,
    bytes_rx: IntCounterVec,
}

impl OtlspMetrics {
    pub fn new_with_registry(registry: &Registry) -> Self {
        Self {
            connections_opened: register_int_counter_vec_with_registry!(
                Opts::new(
                    "connections_opened",
                    "Number of connections that where succesfully opened"
                ),
                &["destination"],
                registry
            )
            .unwrap(),
            connections_closed: register_int_counter_vec_with_registry!(
                Opts::new(
                    "connections_closed",
                    "Number of connections that where closed, possibly with an error"
                ),
                &["destination", "client_init", "error"],
                registry
            )
            .unwrap(),
            connection_errors: register_int_counter_vec_with_registry!(
                Opts::new(
                    "connection_errors",
                    "Count the number of connections that where closed"
                ),
                &["destination", "error"],
                registry
            )
            .unwrap(),
            bytes_tx: register_int_counter_vec_with_registry!(
                Opts::new("bytes_tx", "Number of bytes sent from client to server"),
                &["destination"],
                registry
            )
            .unwrap(),
            bytes_rx: register_int_counter_vec_with_registry!(
                Opts::new("bytes_rx", "Number of bytes sent form server to client"),
                &["destination"],
                registry
            )
            .unwrap(),
        }
    }
}

impl OtlspMetrics {
    pub(crate) fn connection_opened(&self, destination: &str) {
        self.connections_opened
            .with_label_values(&[destination])
            .inc();
    }

    pub(crate) fn connection_closed(
        &self,
        destination: &str,
        client_init: bool,
        error: Option<ErrorKind>,
    ) {
        self.connections_closed
            .with_label_values(&[
                destination,
                &client_init.to_string(),
                &error
                    .map(|err| err.to_string())
                    .unwrap_or("NONE".to_string()),
            ])
            .inc();
    }

    pub(crate) fn connection_error(&self, destination: &str, error_kind: ErrorKind) {
        self.connection_errors
            .with_label_values(&[destination, &error_kind.to_string()])
            .inc()
    }

    pub(crate) fn bytes_send(&self, destination: &str, num_bytes: u64) {
        self.bytes_tx
            .with_label_values(&[destination])
            .inc_by(num_bytes);
    }

    pub(crate) fn bytes_received(&self, destination: &str, num_bytes: u64) {
        self.bytes_rx
            .with_label_values(&[destination])
            .inc_by(num_bytes);
    }
}

impl Default for OtlspMetrics {
    fn default() -> Self {
        Self::new_with_registry(default_registry())
    }
}
