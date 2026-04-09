use crate::{args::Args, conf::Config};
use axum::Router;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod args;
mod conf;
mod otlsp;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv()?;

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

    let router = Router::new()
        //.route(&config.route, get(handle_connection))
        .with_state(config.clone());

    tracing::info!("Serving requests at {}", config.endpoint_addr,);
    axum::serve(listener, router).await.unwrap();
    Ok(())
}
