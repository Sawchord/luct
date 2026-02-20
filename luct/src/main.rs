use crate::{
    args::{Args, get_workdir, log_list_path},
    fetch::fetch_cert_chain,
};
use chrono::DateTime;
use clap::Parser;
use eyre::Context;
use luct_client::{deduplication::RequestDeduplicationClient, reqwest::ReqwestClient};
use luct_core::{log_list::v3::LogList, store::MemoryStore};
use luct_scanner::{LogBuilder, Scanner};
use luct_store::FilesystemStore;
use std::{sync::Arc, time::SystemTime};
use tracing_subscriber::EnvFilter;

mod args;
mod fetch;

const USER_AGENT: &str = concat!(
    "luct-cli/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/Sawchord/luct/)"
);
const LOG_LIST: &str = include_str!("../../luct-extension/luct/assets/log_list.json");

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    if let Ok(env_filter) = EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    }

    let workdir = get_workdir(&args);
    let log_list_path = log_list_path(&args);

    let log_list = log_list_path
        .and_then(|confpath| {
            std::fs::read_to_string(&confpath)
                .inspect_err(|_| {
                    println!(
                        "could not read config path \"{}\", will use default config",
                        confpath.to_str().unwrap()
                    )
                })
                .ok()
        })
        .unwrap_or(LOG_LIST.to_string());

    let log_list: LogList = serde_json::from_str(&log_list)
        .with_context(|| "failed to parse log list json file".to_string())?;
    let logs = log_list.currently_active_logs();
    tracing::info!("Imported {} logs", logs.len());

    let sct_cache = if args.no_cache {
        Box::new(MemoryStore::default()) as _
    } else {
        Box::new(FilesystemStore::new(workdir.join("sct"))) as _
    };

    let sct_report_cache = if args.no_cache {
        Box::new(MemoryStore::default()) as _
    } else {
        Box::new(FilesystemStore::new(workdir.join("sct_report"))) as _
    };

    let client = RequestDeduplicationClient::new(ReqwestClient::new(USER_AGENT));
    let mut scanner = Scanner::new_with_client(sct_cache, sct_report_cache, client);
    tracing::info!("Initialized scanner");

    for log in logs {
        let name = log.description();

        scanner.add_log(
            LogBuilder::new(&log)
                .with_sth_store(FilesystemStore::new(workdir.join("sth").join(name)))
                .with_root_key_store(FilesystemStore::new(workdir.join("roots").join(name))),
        );
    }

    if args.update_sths {
        scanner.update_sths().await?;
    }

    let chain = fetch_cert_chain(&args.source)?;
    println!("Fingerprint: {}", chain.cert().fingerprint_sha256());

    let report = scanner
        .collect_report(Arc::new(chain))
        .await
        .with_context(|| format!("failed to collext leads for {}", args.source))?;
    let report_str = serde_json::to_string_pretty(&report).unwrap();
    println!("Finished report: {}", report_str);

    report
        .evaluate_policy(DateTime::from(SystemTime::now()))
        .map_err(|err| eyre::eyre!(err))?;

    Ok(())
}
