use crate::{args::Args, conf::Config, otlsp::handle_otlsp_connection, state::NodeState};
use axum::{Router, routing::get};
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod args;
mod conf;
mod otlsp;
mod state;

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
    let state = NodeState::new(config)?;

    let router = Router::new();
    let router = if let Some(otlsp_path) = &state.config().otlsp_path {
        tracing::info!("Serving otlsp endpoint at {}", otlsp_path);
        router.route(otlsp_path, get(handle_otlsp_connection))
    } else {
        router
    };

    let router = router.with_state(state);

    axum::serve(listener, router).await.unwrap();
    Ok(())
}
