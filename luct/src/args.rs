use crate::conf::CliConfig;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "luct", version, about, long_about = None)]
pub(crate) struct Args {
    /// The source to check
    #[arg()]
    pub(crate) source: String,

    /// Specify the working directory
    #[arg(short, long, value_name = "FILE")]
    pub(crate) workdir: Option<PathBuf>,

    /// Specify the config directory
    #[arg(short, long, value_name = "FILE")]
    pub(crate) log_list: Option<PathBuf>,

    // TODO: Implement
    /// Reads certificate chain from a file, otherwise fetches the certificate from the URL
    // #[arg(short, long)]
    // pub(crate) file: bool,

    /// Update all logs to the latest signed tree head before checking log inclusions
    #[arg(short, long)]
    pub(crate) update_sths: bool,

    /// Do not use the SCT cache when validating
    #[arg(long)]
    pub(crate) no_cache: bool,

    /// Output the certificate chain to a file
    #[arg(long, value_name = "DESTINATION")]
    pub(crate) output_certificate: Option<PathBuf>,
}

pub(crate) fn get_workdir(args: &Args, config: &CliConfig) -> PathBuf {
    args.workdir
        .clone()
        .unwrap_or_else(|| config.workdir.clone())
}

pub(crate) fn log_list_path(args: &Args, config: &CliConfig) -> Option<PathBuf> {
    args.log_list.clone().or_else(|| config.log_list.clone())
}
