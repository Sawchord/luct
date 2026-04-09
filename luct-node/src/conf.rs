use config::{Config as Conf, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Config {}

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
