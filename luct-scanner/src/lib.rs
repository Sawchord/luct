use futures::future;
use luct_client::{Client, CtClient, CtClientConfig};
use luct_core::{CertificateChain, CtLogConfig, LogId, store::OrderedStore, v1::SignedTreeHead};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

mod lead;
mod log;

use crate::{lead::EmbeddedSct, log::ScannerLog};
pub use lead::{Conclusion, Lead, ScannerConfig, ScannerError};

pub struct Scanner<C> {
    logs: BTreeMap<LogId, ScannerLog<C>>,
    // TODO: CertificateChainStore
    // TODO: Roots denylist
}

#[allow(clippy::type_complexity)]
impl<C: Client + Clone> Scanner<C> {
    pub async fn new_with_client(
        log_configs: BTreeMap<String, (CtLogConfig, Box<dyn OrderedStore<u64, SignedTreeHead>>)>,
        client: C,
    ) -> Self {
        let mut logs = BTreeMap::new();
        for (name, (config, store)) in log_configs {
            let config = CtClientConfig::from(config);
            let client = CtClient::new(config, client.clone());

            let log_id = client.log().log_id().clone();
            let scanner_log = ScannerLog {
                name,
                client,
                sth_store: store,
            };

            logs.insert(log_id, scanner_log);
        }

        Self { logs }
    }

    pub async fn update_sths(&self) -> Result<(), ScannerError> {
        let updates = self
            .logs
            .values()
            .map(|log| log.update_sth())
            .collect::<Vec<_>>();

        future::try_join_all(updates).await?;

        Ok(())
    }

    pub fn collect_leads_pem(&self, data: &str) -> Result<Vec<Lead>, ScannerError> {
        let cert_chain = Arc::new(CertificateChain::from_pem_chain(data)?);
        self.collect_leads(cert_chain)
    }

    pub fn collect_leads(&self, chain: Arc<CertificateChain>) -> Result<Vec<Lead>, ScannerError> {
        // TODO: For embedded SCT, match with the log name immiditately, such that we can print the log

        // TODO: Check that no CA is in the denylist of the scanner
        // TODO: Get OCSP SCT leads
        // TODO: Get revocation list leads
        // TODO: Get DNS CAA leads
        let leads = chain
            .cert()
            .extract_scts_v1()?
            .into_iter()
            .map(|sct| {
                Lead::EmbeddedSct(EmbeddedSct {
                    sct,
                    chain: chain.clone(),
                })
            })
            .collect::<Vec<_>>();

        Ok(leads)
    }
}

impl<C: Client> Scanner<C> {
    pub async fn investigate_lead(&self, lead: &Lead) -> Result<Conclusion, ScannerError> {
        match lead {
            Lead::EmbeddedSct(embedded_sct) => {
                let Some(log) = self.logs.get(&embedded_sct.sct.log_id()) else {
                    return Ok(Conclusion::Inconclusive(format!(
                        "The scanner does not recognize the log {}",
                        embedded_sct.sct.log_id()
                    )));
                };

                log.investigate_embedded_sct(embedded_sct).await
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerBuilder {
    config: ScannerConfig,
    logs: Vec<CtClientConfig>,
}
