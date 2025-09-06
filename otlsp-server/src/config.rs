use serde::{Deserialize, Serialize};
use url::Url;

/// Configure the OTLSP server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Config {
    /// Route at which to mount the proxy
    pub(crate) route: String,

    /// Endpoint
    pub(crate) endpoint: String,

    /// List of URLs, which the connection can be forwarded to
    pub(crate) enabled_urls: Vec<Url>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            route: String::from("/"),
            endpoint: String::from("0.0.0.0:3000"),
            enabled_urls: vec![
                Url::parse("https://127.0.0.1:8080").unwrap(),
                Url::parse("https://localhost:8080").unwrap(),
                Url::parse("https://google.com").unwrap(),
            ],
        }
    }
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
