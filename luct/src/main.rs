use crate::{
    args::{Args, get_workdir, log_list_path},
    fetch::fetch_cert_chain,
};
use clap::Parser;
use eyre::Context;
use futures::future;
use luct_client::reqwest::ReqwestClient;
use luct_core::{log_list::v3::LogList, store::MemoryStore};
use luct_scanner::{LeadResult, LogBuilder, Scanner};
use luct_store::FilesystemStore;
use std::sync::Arc;

mod args;
mod fetch;

const LOG_LIST: &str = include_str!("../../luct-extension/luct/assets/log_list.json");

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
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

    let sct_cache = if args.no_cache {
        Box::new(MemoryStore::default()) as _
    } else {
        Box::new(FilesystemStore::new(workdir.join("sct"))) as _
    };

    let client = ReqwestClient::new();
    let mut scanner = Scanner::new_with_client(sct_cache, client);

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

    let mut leads = scanner
        .collect_leads(Arc::new(chain))
        .with_context(|| format!("failed to collext leads for {}", args.source))?;

    loop {
        for lead in &leads {
            println!("Found a lead: {lead}")
        }

        let investigations: Vec<_> = leads
            .iter()
            .map(async |lead| {
                let result = scanner.investigate_lead(lead).await;
                match &result {
                    LeadResult::Conclusion(conclusion) => {
                        println!("Conclusion: {conclusion}")
                    }
                    LeadResult::FollowUp(_) => (),
                };

                result
            })
            .collect();

        let investigations = future::join_all(investigations).await;

        let follow_ups = investigations
            .into_iter()
            .filter_map(|result| match result {
                LeadResult::Conclusion(_) => None,
                LeadResult::FollowUp(leads) => Some(leads),
            })
            .flatten()
            .collect::<Vec<_>>();

        if follow_ups.is_empty() {
            break;
        } else {
            leads = follow_ups;
        }
    }

    Ok(())
}
