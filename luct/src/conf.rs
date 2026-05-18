use config::{Config as Conf, Environment, File};
use luct_scanner::ScannerConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct CliConfig {
    #[serde(default = "default_workdir")]
    pub(crate) workdir: String,

    #[serde(default = "default_false")]
    pub(crate) validate_cert_chain: bool,

    #[serde(default = "default_none")]
    pub(crate) log_list: Option<String>,

    #[serde(default = "default_none")]
    pub(crate) otlsp_url: Option<String>,

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

fn default_workdir() -> String {
    "~/.luct".to_string()
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
            .add_source(File::with_name("/etc/luct/luct.toml").required(false))
            .add_source(File::with_name("~/.luct/luct.toml").required(false))
            .add_source(File::with_name("luct.toml").required(false))
            .add_source(Environment::with_prefix("LUCT"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}

impl TryFrom<&CliConfig> for ScannerConfig {
    type Error = String;

    fn try_from(config: &CliConfig) -> Result<Self, Self::Error> {
        todo!()
    }
}
