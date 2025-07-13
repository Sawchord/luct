#![allow(dead_code)]
use luct_client::{Client, CtClient, CtClientConfig};
use luct_core::{
    CertificateChain, CertificateError,
    store::Store,
    v1::{SignedCertificateTimestamp, SignedTreeHead},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

pub struct CtScanner<C> {
    logs: Vec<CtScannerLog<C>>,
    // TODO: CertificateChainStore
    // TODO: Roots denylist
}

pub(crate) struct CtScannerLog<C> {
    name: String,
    client: CtClient<C>,
    sht_store: Box<dyn Store<u64, SignedTreeHead>>,
    // TODO: Supported root fingerprints
}

impl<C> CtScanner<C> {
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
            .map(|sct| Lead::EmbeddedSct(sct, chain.clone()))
            .collect::<Vec<_>>();

        Ok(leads)
    }
}

impl<C: Client> CtScanner<C> {
    pub async fn investigate_lead(&self, _lead: &Lead) -> Result<Conclusion, ScannerError> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtScannerBuilder {
    config: CtScannerConfig,
    logs: Vec<CtClientConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtScannerConfig {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lead {
    EmbeddedSct(SignedCertificateTimestamp, Arc<CertificateChain>),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ScannerError {
    #[error("Certificate error: {0}")]
    CertificateError(#[from] CertificateError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Conclusion {
    Safe(String),
    Unsafe(String),
}
