use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "luct", version, about, long_about = None)]
pub(crate) struct Args {
    /// The source to check
    #[arg()]
    source: String,

    /// Specify the working directory
    #[arg(short, long, value_name = "FILE")]
    workdir: Option<PathBuf>,

    /// Specify the config directory
    #[arg(short, long, value_name = "FILE")]
    confdir: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// If set, reads certificate chain from a file, otherwise fetches the certificate from the URL
    #[arg(short, long)]
    file: bool,
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

pub(crate) fn get_confpath(args: &Args, workdir: &Path) -> PathBuf {
    args.workdir
        .clone()
        .unwrap_or_else(|| workdir.join("logs.toml"))
}
