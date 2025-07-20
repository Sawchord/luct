use luct_client::{Client, CtClientConfig};
use luct_core::{CertificateChain, CertificateError, LogId};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use thiserror::Error;

mod lead;
mod log;

pub use lead::{Conclusion, Lead};

use crate::{lead::EmbeddedSct, log::ScannerLog};

pub struct Scanner<C> {
    logs: BTreeMap<LogId, ScannerLog<C>>,
    // TODO: CertificateChainStore
    // TODO: Roots denylist
}

impl<C> Scanner<C> {
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
    pub async fn investigate_lead(&self, _lead: &Lead) -> Result<Conclusion, ScannerError> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerBuilder {
    config: ScannerConfig,
    logs: Vec<CtClientConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScannerConfig {}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScannerError {
    #[error("Certificate error: {0}")]
    CertificateError(#[from] CertificateError),
}
