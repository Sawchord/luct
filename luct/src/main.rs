// TODO: Get the certificate chain of the request
// TODO: Scan for leads
// TODO: Investigate the leads and print the results

use crate::{
    args::{Args, get_confpath, get_workdir},
    fetch::fetch_cert_chain,
};
use clap::Parser;

mod args;
mod fetch;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let workdir = get_workdir(&args);
    let confpath = get_confpath(&args, &workdir);

    let chain = fetch_cert_chain(&args.source)?;

    println!("{confpath:?}");
    println!("Hello world");

    Ok(())
}
