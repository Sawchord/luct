//! Wrapper around [`Scanner`](CtScanner) to be used in a javascript environment.

use crate::store::BrowserStore;
use js_sys::{Array, Uint8Array};
use luct_client::reqwest::ReqwestClient;
use luct_core::{CertificateChain, CtLogConfig, v1::SignedCertificateTimestamp};
use luct_scanner::{
    Conclusion as CtConclusion, Lead as CtLead, LeadResult as CtLeadResult, Log,
    Scanner as CtScanner,
};
use std::{collections::BTreeMap, sync::Arc};
use url::Url;
use wasm_bindgen::prelude::wasm_bindgen;

mod store;

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

        let client = luct_client::reqwest::ReqwestClient::new();
        let sct_cache = Box::new(
            BrowserStore::<[u8; 32], SignedCertificateTimestamp>::new_local_store(
                "sct".to_string(),
            )
            .expect("Failed to inistalize SCT cache"),
        ) as _;
        let mut scanner = CtScanner::new_with_client(sct_cache, client);

        for (name, config) in log_configs {
            scanner.add_log(Log {
                name: name.clone(),
                config,
                sth_store: Box::new(
                    BrowserStore::new_local_store(format!("sth/{name}"))
                        .expect("Failed to initialize STH store"),
                ) as _,
                root_keys: Box::new(
                    BrowserStore::new_local_store(format!("roots/{name}"))
                        .expect("Failed to initialize allowed roots fingerprint store"),
                ),
            });
        }

        log("Initialized scanner");
        Ok(Scanner(scanner))
    }

    #[wasm_bindgen]
    pub fn collect_leads(&self, url: String, leads: Array) -> Result<Vec<Lead>, String> {
        let url = Url::parse(&url).map_err(|err| format!("{err}"))?;
        if self
            .0
            .logs()
            .any(|log| log.config().url().domain() == url.domain())
        {
            log("Skipping request to log itself to prevent recursion");
            return Ok(vec![]);
        }

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
