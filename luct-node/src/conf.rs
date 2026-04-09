use config::{Config as Conf, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Config {
    /// Endpoint address
    pub(crate) endpoint_addr: String,

    /// Path to the log list to use in luct-node
    pub(crate) log_list: String,

    /// Path at which to serve the oblivious TLS proxy
    pub(crate) otlsp_path: Option<String>,
}

impl Config {
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
