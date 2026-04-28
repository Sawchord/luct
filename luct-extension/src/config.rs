use luct_scanner::ScannerConfig;
use serde::{Deserialize, Serialize};
use url::Url;
use web_time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    #[serde(default = "default_true")]
    validate_cert_chain: bool,

    #[serde(default = "otlsp_url")]
    otlsp_url: Option<String>,

    #[serde(default = "sth_freshness_threshold")]
    sth_freshness_threshold: u64,

    #[serde(default = "sth_update_threshold")]
    sth_update_threshold: u64,

    #[serde(default = "default_false")]
    debug_output: bool,
}

fn otlsp_url() -> Option<String> {
    // Some("https://node.luct.dev/otlsp".to_string())
    None
}

fn sth_freshness_threshold() -> u64 {
    24 * 60 * 60
}

fn sth_update_threshold() -> u64 {
    8 * 60 * 60
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

impl TryFrom<&ExtensionConfig> for ScannerConfig {
    type Error = String;

    fn try_from(config: &ExtensionConfig) -> Result<Self, Self::Error> {
        let otlsp_url = config
            .otlsp_url
            .as_ref()
            .map(|url| Url::parse(url))
            .transpose()
            .map_err(|err| err.to_string())?;

        let config = ScannerConfig::builder()
            .validate_cert_chain(config.validate_cert_chain)
            .otlsp_url(otlsp_url)
            .sth_freshness_threshold(Duration::from_secs(config.sth_freshness_threshold))
            .sth_update_threshold(Duration::from_secs(config.sth_update_threshold))
            .build()
            .map_err(|err| err.to_string())?;

        Ok(config)
    }
}
