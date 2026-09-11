#![forbid(unsafe_code)]

use crate::{
    args::Args,
    checkpoint::{CheckpointerImpl, handle_checkpoint_request, handle_toc_request},
    conf::Config,
    metrics::handle_metrics_request,
    otlsp::handle_otlsp_connection,
    state::NodeState,
};
use axum::{Router, routing::get};
use clap::Parser;
use luct_client::{deduplication::RequestDeduplicationClient, reqwest::ReqwestClient};
use luct_store::FilesystemStore;
use tracing_subscriber::EnvFilter;

mod args;
mod checkpoint;
mod conf;
mod metrics;
mod otlsp;
mod state;

const USER_AGENT: &str = concat!(
    "luct-checkpointer/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/Sawchord/luct/)"
);

#[derive(Debug, Clone)]
struct NodeCheckpointerImpl;

impl CheckpointerImpl for NodeCheckpointerImpl {
    type CheckpointFetcher = RequestDeduplicationClient<ReqwestClient>;
    type CheckpointStore = FilesystemStore<u64, String>;
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    if dotenv::dotenv().is_ok() {
        tracing::info!("Loaded .env directory");
    }

    let _args = Args::parse();

    if let Ok(env_filter) = EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    }

    let config = Config::parse()?;

    let listener = tokio::net::TcpListener::bind(config.endpoint_addr.clone())
        .await
        .unwrap();

    tracing::info!("Serving requests at {}", config.endpoint_addr);
    let state = NodeState::new(config).await?;
    let router = Router::new();

    let router = if let Some(metrics_path) = &state.config().metrics_path {
        tracing::info!("Serving metrics endpoint at {}", metrics_path);
        router.route(metrics_path, get(handle_metrics_request))
    } else {
        router
    };

    let router = if let Some(otlsp_path) = &state.config().otlsp_path {
        tracing::info!("Serving otlsp endpoint at {}", otlsp_path);
        router.route(otlsp_path, get(handle_otlsp_connection))
    } else {
        router
    };

    let router = if let Some(checkpoint_path) = &state.config().checkpoint_path {
        tracing::info!("Serving checkpoint endpoint at {}", checkpoint_path);

        router
            .route(
                &format!("{}/{{log_id}}", checkpoint_path),
                get(handle_toc_request),
            )
            .route(
                &format!("{}/{{log_id}}/{{tree_size}}", checkpoint_path),
                get(handle_checkpoint_request),
            )
    } else {
        router
    };

    let router = router.with_state(state);

    axum::serve(listener, router).await.unwrap();
    Ok(())
}
