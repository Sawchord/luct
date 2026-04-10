use crate::conf::Config;
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::Response,
};
use axum_macros::debug_handler;
use eyre::Context;
use luct_core::log_list::v3::LogList;
use otlsp_server::Destination;
use std::collections::BTreeSet;
use url::Url;

impl Config {
    /// Extract all urls the otlsp service needs to enable
    pub(crate) fn get_otlsp_urls(&self) -> eyre::Result<Vec<Url>> {
        let logs = std::fs::read_to_string(&self.log_list)
            .with_context(|| format! {"Could not find log list file at {}", self.log_list})?;
        let logs: LogList =
            serde_json::from_str(&logs).with_context(|| "Failed to parse log list")?;
        let logs = logs.currently_active_logs();
        tracing::info!("Imported {} logs", logs.len());

        let urls: BTreeSet<Url> = logs
            .iter()
            .map(|log| log.config().fetch_url().clone())
            .chain(
                logs.iter()
                    .filter_map(|log| log.config().tile_url().clone()),
            )
            .collect();

        tracing::info!("Enabled {} urls", urls.len());

        Ok(urls.into_iter().collect())
    }
}

#[debug_handler]
pub(crate) async fn handle_otlsp_connection(
    config: State<(Config, Vec<Url>)>,
    destination: Query<Destination>,
    ws: WebSocketUpgrade,
) -> Response {
    todo!()
}
