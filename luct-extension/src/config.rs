use crate::store::browser_local_store;
use luct_scanner::ScannerConfig;
use serde::{Deserialize, Serialize};
use url::Url;
use web_time::Duration;

/// Loads the config from the local store
///
/// If no settings exist, it will create some
pub fn load_config() -> Result<ExtensionConfig, String> {
    let store = browser_local_store()?;

    let settings = match store
        .get_item("settings")
        .map_err(|err| err.as_string().unwrap())?
    {
        Some(settings) => serde_json::from_str::<ExtensionConfig>(&settings),
        None => {
            tracing::info!("Could not find a config. Initalizing with default");
            let settings = serde_json::from_str::<ExtensionConfig>("{}").unwrap();
            store
                .set_item("settings", &serde_json::to_string(&settings).unwrap())
                .map_err(|err| err.as_string().unwrap())?;
            Ok(settings)
        }
    };

    settings.map_err(|err| err.to_string())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    fn default_config() {
        serde_json::from_str::<ExtensionConfig>("{}").unwrap();
    }

    #[wasm_bindgen_test]
    fn initalize_config() {
        load_config().unwrap();
    }
}
