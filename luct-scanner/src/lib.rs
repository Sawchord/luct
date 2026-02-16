use crate::{lead::EmbeddedSct, log::ScannerLog};
use futures::future;
use luct_client::{Client, ClientError};
use luct_core::{
    CertificateChain, CertificateError, CheckSeverity, CtLog, CtLogConfig, LogId, Severity,
    store::Store, tiling::TilingError, v1::SignedCertificateTimestamp,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;
pub use {
    lead::{Conclusion, Lead, LeadResult, ScannerConfig},
    log::builder::LogBuilder,
};

type HashOutput = [u8; 32];

mod lead;
mod log;

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
        Box::new(self.logs.values().map(|val| val.client().log()))
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

    pub fn add_log(&mut self, log: LogBuilder) -> &mut Self {
        let scanner_log = log.build(&self.client);
        let log_id = scanner_log.client().log().log_id().clone();

        self.logs.insert(log_id, scanner_log);
        self
    }
}

impl<C: Client> Scanner<C> {
    pub async fn update_sths(&self) -> Result<(), ScannerError> {
        let updates = self
            .logs
            .values()
            .map(|log| log.update_sth())
            .collect::<Vec<_>>();

        future::try_join_all(updates).await?;

        Ok(())
    }

    pub async fn update_sth(&self, log_name: &str) -> Result<(), ScannerError> {
        match self
            .logs
            .values()
            .find(|log| log.client().log().description() == log_name)
        {
            Some(log) => Ok(log.update_sth().await?),
            None => {
                tracing::warn!("Failed to find log {} to update", log_name);
                Ok(())
            }
        }
    }

    /// Collect the [`Leads`](Lead) from a [`CertificateChain`], encoded as a series
    /// of PEM encoded certificates.
    pub fn collect_leads_pem(&self, data: &str) -> Result<Vec<Lead>, ScannerError> {
        let cert_chain = Arc::new(CertificateChain::from_pem_chain(data)?);
        cert_chain.verify_chain()?;
        self.collect_leads(cert_chain)
    }

    /// Collect the [`Leads`](Lead) from a [`CertificateChain`]
    pub fn collect_leads(&self, chain: Arc<CertificateChain>) -> Result<Vec<Lead>, ScannerError> {
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

    async fn investigate_lead_impl(&self, lead: &Lead) -> Result<LeadResult, ScannerError> {
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

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScannerError {
    #[error("Invalid certificate: {0}")]
    CertificateError(#[from] CertificateError),

    #[error("HTTP client error {0}")]
    ClientError(#[from] ClientError),

    #[error("Failed to construct proof from tiles {0}")]
    TilingError(#[from] TilingError),
}

impl CheckSeverity for ScannerError {
    fn severity(&self) -> Severity {
        match self {
            ScannerError::CertificateError(err) => err.severity(),
            ScannerError::ClientError(err) => err.severity(),
            ScannerError::TilingError(_) => Severity::Unsafe,
        }
    }
}
