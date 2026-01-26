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
    pub(crate) confdir: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) debug: u8,

    /// Reads certificate chain from a file, otherwise fetches the certificate from the URL
    #[arg(short, long)]
    pub(crate) file: bool,

    /// Update all logs to the latest signed tree head before checking log inclusions
    #[arg(short, long)]
    pub(crate) update_sths: bool,

    /// Do not use the SCT cache when validating
    #[arg(long)]
    pub(crate) no_cache: bool,
}

pub(crate) fn get_workdir(args: &Args) -> PathBuf {
    args.workdir.clone().unwrap_or_else(|| {
        std::env::var("LUCT_WORKDIR").map(PathBuf::from).unwrap_or(
            std::env::home_dir()
                .expect("Home directory not set")
                .join(".luct"),
        )
    })
}

pub(crate) fn log_list_path(args: &Args) -> Option<PathBuf> {
    args.confdir.clone()
}
