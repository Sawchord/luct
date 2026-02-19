use crate::{
    lead::EmbeddedSct,
    log::ScannerLog,
    report::{SctReport, SthReport},
};
use futures::future::{self, join_all};
use luct_client::{Client, ClientError};
use luct_core::{
    CertificateChain, CertificateError, CheckSeverity, CtLog, CtLogConfig, LogId, Severity,
    store::Store, tiling::TilingError, v1::SignedCertificateTimestamp,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;
use web_time::SystemTime;
pub use {
    lead::{Conclusion, Lead, LeadResult, ScannerConfig},
    log::builder::LogBuilder,
    report::Report,
    utils::Validated,
};

type HashOutput = [u8; 32];

mod lead;
mod log;
mod report;
mod utils;

pub struct Scanner<C> {
    logs: BTreeMap<LogId, ScannerLog<C>>,
    sct_cache: Box<dyn Store<HashOutput, Validated<SignedCertificateTimestamp>>>,
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
        sct_cache: Box<dyn Store<HashOutput, Validated<SignedCertificateTimestamp>>>,
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

    pub async fn collect_report_pem(&self, data: &str) -> Result<Report, ScannerError> {
        let cert_chain = Arc::new(CertificateChain::from_pem_chain(data)?);
        cert_chain.verify_chain()?;
        self.collect_report(cert_chain).await
    }

    pub async fn collect_report(
        &self,
        chain: Arc<CertificateChain>,
    ) -> Result<Report, ScannerError> {
        let cert = chain.cert();
        let (not_before, not_after) = cert.get_validity();

        let embedded_scts = cert.extract_scts_v1()?;

        let scts = join_all(
            embedded_scts
                .into_iter()
                .map(|sct| self.collect_embedded_sct_report(sct, &chain)),
        )
        .await;

        Ok(Report {
            ca_name: chain.root().get_issuer_name(),
            not_before: not_before.into(),
            not_after: not_after.into(),
            scts,
        })
    }

    pub(crate) async fn collect_embedded_sct_report(
        &self,
        sct: SignedCertificateTimestamp,
        chain: &Arc<CertificateChain>,
    ) -> SctReport {
        // TODO: Use a report cache, such that we can just enter the data and look it up
        // if let Some(sct) = self.sct_cache.get(&sct.hash) {
        //     todo!()
        // };

        let report = SctReport::new();

        // Find the log this sct belongs to
        let Some(log) = self.logs.get(&sct.log_id()) else {
            return report.error_description(format!("No log with id {} known", sct.log_id()));
        };
        let report = report.log_name(log.client().log().description().to_string());

        // Validate the signature
        if let Err(err) = log.client().log().validate_sct_v1(chain, &sct, true) {
            return report.error_description(err.to_string());
        };
        let report = report.signature_validation_time(SystemTime::now().into());

        // TODO: Use oldest valid sth for the inclusion proof, not last. Then only add last_sth after caching

        // Get the last sth
        let last_sth = match log.latest_sth().await {
            Err(err) => {
                return report.error_description(err.to_string());
            }
            Ok(sth) => sth,
        };
        let report = report.last_sth(SthReport::from(&last_sth));

        let leaf = match chain.as_leaf_v1(&sct, true) {
            Err(err) => {
                return report.error_description(err.to_string());
            }
            Ok(leaf) => leaf,
        };

        // Check inclusion
        if let Err(err) = log.check_sct_inclusion(&sct, &last_sth, &leaf).await {
            return report.error_description(err.to_string());
        };

        // TODO: Cache final report

        report.inclusion_proof(SthReport::from(&last_sth))
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
