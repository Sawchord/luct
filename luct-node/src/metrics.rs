use crate::state::NodeState;
use axum::{extract::State, response::Response};
use prometheus::TextEncoder;

pub(crate) async fn handle_metrics_request(_state: State<NodeState>) -> Response {
    tracing::debug!("Serving metrics");

    let metrics = prometheus::gather();
    match TextEncoder::new().encode_to_string(&metrics) {
        Err(err) => Response::builder()
            .status(500)
            .body(err.to_string().into())
            .unwrap(),
        Ok(body) => Response::builder().status(200).body(body.into()).unwrap(),
    }
}
