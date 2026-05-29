#![forbid(unsafe_code)]

use crate::{
    args::{Args, get_workdir, log_list_path},
    conf::CliConfig,
    fetch::fetch_cert_chain,
};
use chrono::DateTime;
use clap::Parser;
use eyre::Context;
use luct_client::deduplication::RequestDeduplicationClient;
use luct_core::{
    Fingerprint,
    log_list::v3::LogList,
    store::{MemoryStore, StoreRead},
    v1::SignedTreeHead,
};
use luct_otlsp::{OtlspClient, OtlspClientConfig};
use luct_scanner::{Report, Scanner, ScannerConfig, ScannerImpl, Validated};
use luct_store::{FilesystemStore, StoreSwitch};
use std::{sync::Arc, time::SystemTime};
use tracing_subscriber::EnvFilter;

mod args;
mod conf;
mod fetch;

const USER_AGENT: &str = concat!(
    "luct-cli/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/Sawchord/luct/)"
);
const LOG_LIST: &str = include_str!("../log_list.json");

struct CliScannerImpl;

impl ScannerImpl for CliScannerImpl {
    type Client = RequestDeduplicationClient<OtlspClient>;
    type ReportStore =
        StoreSwitch<MemoryStore<Fingerprint, Report>, FilesystemStore<Fingerprint, Report>>;
    type SthStore = FilesystemStore<u64, Validated<SignedTreeHead>>;
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let config = CliConfig::parse()?;
    let args = Args::parse();

    if let Ok(env_filter) = EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    }
    let workdir = get_workdir(&args, &config);
    let log_list_path = log_list_path(&args, &config);
    tracing::debug!("Workdir: {:?}, log list path: {:?}", workdir, log_list_path);

    let log_list = match log_list_path {
        None => LOG_LIST.to_string(),
        Some(log_list) => {
            std::fs::read_to_string(&log_list).with_context(|| "failed to read log_list.json")?
        }
    };

    let log_list: LogList = serde_json::from_str(&log_list)
        .with_context(|| "failed to parse log list json file".to_string())?;
    let logs = log_list.currently_active_logs();
    tracing::info!("Imported {} logs", logs.len());

    let report_cache = if args.no_cache {
        StoreSwitch::A(MemoryStore::default())
    } else {
        let store = StoreSwitch::B(FilesystemStore::new(workdir.join("report")));
        tracing::debug!("Loaded report store with {} cached reports", store.len());
        store
    };

    let scanner_config = ScannerConfig::try_from(&config).map_err(|err| eyre::eyre!(err))?;
    let client_config = OtlspClientConfig::try_from(&config).map_err(|err| eyre::eyre!(err))?;

    let client = RequestDeduplicationClient::new(OtlspClient::new(client_config));
    let time_source = || DateTime::from(SystemTime::now());

    let mut scanner =
        Scanner::<CliScannerImpl>::new(scanner_config, report_cache, client, time_source);
    tracing::info!("Initialized scanner");

    for log in logs {
        let name = log.description();
        scanner.add_log(&log, FilesystemStore::new(workdir.join("sth").join(name)));
    }

    if args.update_sths {
        scanner.refresh_all_logs().await?;
    }

    let chain = fetch_cert_chain(&args.source)?;
    println!("Fingerprint: {}", chain.cert().fingerprint_sha256());

    if let Some(destination) = args.output_certificate {
        let chain_str = chain.as_pem_chain();
        std::fs::write(destination, chain_str).expect("failed to output pem chain")
    }

    let report = scanner
        .collect_report(Arc::new(chain))
        .await
        .with_context(|| format!("failed to collext leads for {}", args.source))?;

    let report = serde_json::to_string_pretty(&report).unwrap();
    println!("Finished report: {}", report);

    Ok(())
}
