//! Wrapper around [`Scanner`](CtScanner) to be used in a javascript environment.

use js_sys::{Array, Uint8Array};
use luct_client::reqwest::ReqwestClient;
use luct_core::{CertificateChain, CtLogConfig, store::MemoryStore, v1::SignedTreeHead};
use luct_scanner::{
    Conclusion as CtConclusion, Lead as CtLead, LeadResult as CtLeadResult, Scanner as CtScanner,
};
use std::{collections::BTreeMap, sync::Arc};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Scanner(CtScanner<ReqwestClient>);

#[wasm_bindgen]
impl Scanner {
    #[wasm_bindgen(constructor)]
    pub fn new(config: String) -> Result<Self, String> {
        let log_configs: BTreeMap<String, CtLogConfig> =
            toml::from_str(&config).map_err(|err| format!("{err}"))?;

        let log_configs = log_configs
            .into_iter()
            .map(|(name, config)| {
                (
                    name.clone(),
                    (
                        config,
                        Box::new(MemoryStore::<u64, SignedTreeHead>::default()) as _,
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>();

        let client = luct_client::reqwest::ReqwestClient::new();
        let scanner = CtScanner::new_with_client(log_configs, client);

        log("Initialized scanner");
        Ok(Scanner(scanner))
    }

    #[wasm_bindgen]
    pub fn collect_leads(&self, leads: Array) -> Result<Vec<Lead>, String> {
        let cert_chain_bytes = leads
            .to_vec()
            .into_iter()
            .map(|value| Uint8Array::from(value).to_vec())
            .collect::<Vec<_>>();

        let cert_chain =
            CertificateChain::from_der_chain(&cert_chain_bytes).map_err(|err| format!("{err}"))?;
        //log(&format!("{cert_chain:?}"));

        let leads = self
            .0
            .collect_leads(Arc::new(cert_chain))
            .map_err(|err| format!("{err}"))?;
        //log(&format!("{leads:?}"));

        //log("collected leads");
        Ok(leads.into_iter().map(Lead).collect())
    }

    #[wasm_bindgen]
    pub async fn investigate_lead(&self, lead: Lead) -> LeadResult {
        //log("investigating lead");
        LeadResult(self.0.investigate_lead(&lead.0).await)
    }
}

#[wasm_bindgen]
pub struct Lead(CtLead);

#[wasm_bindgen]
impl Lead {
    #[wasm_bindgen]
    pub fn description(&self) -> String {
        format!("{}", self.0)
    }
}

#[wasm_bindgen]
pub struct LeadResult(CtLeadResult);

#[wasm_bindgen]
impl LeadResult {
    #[wasm_bindgen]
    pub fn conclusion(&self) -> Option<Conclusion> {
        match &self.0 {
            CtLeadResult::Conclusion(conclusion) => Some(Conclusion(conclusion.clone())),
            CtLeadResult::FollowUp(_) => None,
        }
    }

    #[wasm_bindgen]
    pub fn follow_up(self) -> Vec<Lead> {
        match self.0 {
            CtLeadResult::Conclusion(_) => vec![],
            CtLeadResult::FollowUp(leads) => leads.into_iter().map(Lead).collect(),
        }
    }
}

#[wasm_bindgen]
pub struct Conclusion(CtConclusion);

#[wasm_bindgen]
impl Conclusion {
    #[wasm_bindgen]
    pub fn description(&self) -> String {
        format!("{}", self.0)
    }
}
