use crate::{lead::EmbeddedSct, log::ScannerLog};
use futures::future;
use luct_client::{Client, ClientError, CtClient};
use luct_core::{
    CertificateChain, CtLog, CtLogConfig, LogId,
    store::{MemoryStore, Store},
    v1::SignedCertificateTimestamp,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
pub use {
    lead::{Conclusion, Lead, LeadResult, ScannerConfig},
    log::Log,
};

type HashOutput = [u8; 32];

mod lead;
mod log;
mod tiling;

pub struct Scanner<C> {
    logs: BTreeMap<LogId, ScannerLog<C>>,
    sct_cache: Box<dyn Store<HashOutput, SignedCertificateTimestamp>>,
    client: C,
    // TODO: CertificateChainStore
    // TODO: Roots denylist
}

#[allow(clippy::type_complexity)]
impl<C: Client + Clone> Scanner<C> {
    pub fn logs<'a>(&'a self) -> Box<dyn Iterator<Item = &'a CtLog> + 'a> {
        Box::new(self.logs.values().map(|val| val.client.log()))
    }

    pub fn new_with_client(
        sct_cache: Box<dyn Store<HashOutput, SignedCertificateTimestamp>>,
        client: C,
    ) -> Self {
        Self {
            logs: BTreeMap::new(),
            sct_cache,
            client,
        }
    }

    pub fn add_log(&mut self, log: Log) -> &mut Self {
        let client = CtClient::new(log.config, self.client.clone());
        let log_id = client.log().log_id().clone();
        let scanner_log = ScannerLog {
            name: log.name,
            client,
            sth_store: log
                .sth_store
                .unwrap_or_else(|| Box::new(MemoryStore::default())),
            root_keys: log
                .root_keys
                .unwrap_or_else(|| Box::new(MemoryStore::default())),
        };

        self.logs.insert(log_id, scanner_log);
        self
    }
}

impl<C: Client> Scanner<C> {
    pub async fn update_sths(&self) -> Result<(), ClientError> {
        let updates = self
            .logs
            .values()
            .map(|log| log.update_sth())
            .collect::<Vec<_>>();

        future::try_join_all(updates).await?;

        Ok(())
    }

    /// Collect the [`Leads`](Lead) from a [`CertificateChain`], encoded as a series
    /// of PEM encoded certificates.
    pub fn collect_leads_pem(&self, data: &str) -> Result<Vec<Lead>, ClientError> {
        let cert_chain = Arc::new(CertificateChain::from_pem_chain(data)?);
        cert_chain.verify_chain()?;
        self.collect_leads(cert_chain)
    }

    /// Collect the [`Leads`](Lead) from a [`CertificateChain`]
    pub fn collect_leads(&self, chain: Arc<CertificateChain>) -> Result<Vec<Lead>, ClientError> {
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

    pub async fn investigate_lead(&self, lead: &Lead) -> LeadResult {
        let result = self.investigate_lead_impl(lead).await;

        match result {
            Ok(result) => result,
            Err(err) => LeadResult::Conclusion(err.into()),
        }
    }

    async fn investigate_lead_impl(&self, lead: &Lead) -> Result<LeadResult, ClientError> {
        match lead {
            Lead::EmbeddedSct(embedded_sct) => {
                let Some(log) = self.logs.get(&embedded_sct.sct.log_id()) else {
                    return Ok(LeadResult::Conclusion(Conclusion::Inconclusive(format!(
                        "The scanner does not recognize the log {}",
                        embedded_sct.sct.log_id()
                    ))));
                };

                log.investigate_embedded_sct(embedded_sct, self)
                    .await
                    .map(LeadResult::Conclusion)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerBuilder {
    config: ScannerConfig,
    logs: Vec<CtLogConfig>,
}
