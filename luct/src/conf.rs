use config::{Config as Conf, Environment, File};
use luct_otlsp::OtlspClientConfig;
use luct_scanner::ScannerConfig;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};
use url::Url;

use crate::USER_AGENT;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct CliConfig {
    #[serde(default = "default_workdir")]
    pub(crate) workdir: PathBuf,

    #[serde(default = "default_false")]
    pub(crate) validate_cert_chain: bool,

    #[serde(default = "default_none")]
    pub(crate) log_list: Option<PathBuf>,

    #[serde(default = "default_none")]
    pub(crate) otlsp_url: Option<Url>,

    #[serde(default = "default_sth_freshness_threshold")]
    pub(crate) sth_freshness_threshold: u64,

    #[serde(default = "default_sth_update_threshold")]
    pub(crate) sth_update_threshold: u64,
}

fn default_false() -> bool {
    false
}

fn default_none<T>() -> Option<T> {
    None
}

fn default_workdir() -> PathBuf {
    std::env::home_dir()
        .expect("Home directory not set")
        .join(".luct")
}

fn default_sth_freshness_threshold() -> u64 {
    24 * 60 * 60
}

fn default_sth_update_threshold() -> u64 {
    8 * 60 * 60
}

impl CliConfig {
    pub(crate) fn parse() -> eyre::Result<Self> {
        let config = Conf::builder()
            .add_source(File::from(PathBuf::from("/etc/luct/luct.toml")).required(false))
            .add_source(
                File::from(
                    std::env::home_dir()
                        .expect("Home directory not set")
                        .join(".luct/luct.toml"),
                )
                .required(false),
            )
            .add_source(File::from(PathBuf::from("luct.toml")).required(false))
            .add_source(Environment::with_prefix("LUCT"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}

impl TryFrom<&CliConfig> for ScannerConfig {
    type Error = String;

    fn try_from(config: &CliConfig) -> Result<Self, Self::Error> {
        let config = ScannerConfig::builder()
            .validate_cert_chain(config.validate_cert_chain)
            .otlsp_url(config.otlsp_url.clone())
            .sth_freshness_threshold(Duration::from_secs(config.sth_freshness_threshold))
            .sth_update_threshold(Duration::from_secs(config.sth_update_threshold))
            .build()
            .map_err(|err| err.to_string())?;

        Ok(config)
    }
}

impl TryFrom<&CliConfig> for OtlspClientConfig {
    type Error = String;

    fn try_from(config: &CliConfig) -> Result<Self, Self::Error> {
        let config = OtlspClientConfig::builder()
            .agent(USER_AGENT.to_string())
            .proxy_url(config.otlsp_url.clone())
            .build()
            .map_err(|err| err.to_string())?;

        Ok(config)
    }
}
