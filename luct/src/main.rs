// TODO: Use tracing to output sources

use std::{collections::BTreeMap, sync::Arc};

use crate::{
    args::{Args, get_confpath, get_workdir},
    fetch::fetch_cert_chain,
};
use clap::Parser;
use eyre::Context;
use futures::future;
use luct_core::{CtLogConfig, v1::SignedTreeHead};
use luct_scanner::{LeadResult, Scanner};
use luct_store::FilesystemStore;

mod args;
mod fetch;

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let workdir = get_workdir(&args);
    let confpath = get_confpath(&args, &workdir);

    let config = std::fs::read_to_string(&confpath).with_context(|| {
        format!(
            "could not read config path \"{}\"",
            confpath.to_str().unwrap()
        )
    })?;
    let log_configs: BTreeMap<String, CtLogConfig> = toml::from_str(&config)?;

    let log_configs = log_configs
        .into_iter()
        .map(|(name, config)| {
            (
                name.clone(),
                (
                    config,
                    Box::new(FilesystemStore::<u64, SignedTreeHead>::new(
                        workdir.join("sth").join(name),
                    )) as _,
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let client = luct_client::reqwest::ReqwestClient::new();
    let scanner = Scanner::new_with_client(log_configs, client).await;

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
                match scanner.investigate_lead(lead).await {
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
