use crate::{conf::Config, state::NodeState};
use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::Response,
};
use axum_macros::debug_handler;
use eyre::Context;
use luct_core::log_list::v3::LogList;
use otlsp_server::{Destination, handle_connection};
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
            // Remove the paths, we need to have access to the entire paths
            .map(|mut url| {
                url.set_path("");
                url
            })
            .collect();

        tracing::info!("Enabled {} urls", urls.len());

        Ok(urls.into_iter().collect())
    }
}

#[debug_handler]
pub(crate) async fn handle_otlsp_connection(
    config: State<NodeState>,
    destination: Query<Destination>,
    ws: WebSocketUpgrade,
) -> Response {
    tracing::trace!("Received a new connection request to {:?}", destination);

    let has_access = move |destination: Url| {
        config
            .otlsp_urls()
            .iter()
            .any(|url| is_valid_destination(url, &destination))
    };

    handle_connection(destination.dst().clone(), ws, has_access).await
}

/// Test whether the [`Url`] `dst` is valid against the [`Url`] `dst`
///
/// A destination is valid, if it has the same:
/// - Protocol
/// - Domain
/// - Port
///
/// well as the path of `config` is a prefix of the path of `dst`
pub(crate) fn is_valid_destination(config: &Url, dst: &Url) -> bool {
    config.scheme() == dst.scheme()
        && config.domain() == dst.domain()
        && config.port() == dst.port()
        && dst.path().starts_with(config.path())
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_valid_destination() {
        // Test that different schemes don't match
        assert!(!is_valid_destination(
            &Url::parse("http://example.com").unwrap(),
            &Url::parse("https://example.com").unwrap()
        ));

        // Test that different hosts don't match
        assert!(!is_valid_destination(
            &Url::parse("https://example.org").unwrap(),
            &Url::parse("https://example.com").unwrap()
        ));

        // Test that different ports don't match
        assert!(!is_valid_destination(
            &Url::parse("https://example.com:8080").unwrap(),
            &Url::parse("https://example.com:3000").unwrap()
        ));

        // Test that different paths don't match
        assert!(!is_valid_destination(
            &Url::parse("https://example.com/path").unwrap(),
            &Url::parse("https://example.com/other_path").unwrap()
        ));

        // Test that subpaths are included
        assert!(is_valid_destination(
            &Url::parse("https://example.com").unwrap(),
            &Url::parse("https://example.com/").unwrap()
        ));

        assert!(is_valid_destination(
            &Url::parse("https://example.com/path").unwrap(),
            &Url::parse("https://example.com/path/subpath").unwrap()
        ));
    }
}
