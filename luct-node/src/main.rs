use crate::{args::Args, conf::Config};
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod args;
mod conf;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    dotenv::dotenv()?;

    let _args = Args::parse();
    let _conf = Config::parse()?;

    if let Ok(env_filter) = EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    }

    Ok(())
}
