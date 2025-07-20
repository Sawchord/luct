// TODO: Get the certificate chain of the request
// TODO: Scan for leads
// TODO: Investigate the leads and print the results

use crate::args::{Args, get_confpath, get_workdir};
use clap::Parser;

mod args;

fn main() {
    let args = Args::parse();
    let workdir = get_workdir(&args);
    let confpath = get_confpath(&args, &workdir);

    println!("{confpath:?}");
    println!("Hello world");
}
