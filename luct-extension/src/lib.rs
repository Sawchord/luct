//! Wrapper around [`Scanner`](CtScanner) to be used in a javascript environment.

use crate::store::BrowserStore;
use chrono::DateTime;
use js_sys::{Array, Uint8Array};
use luct_client::deduplication::RequestDeduplicationClient;
use luct_core::{CertificateChain, Fingerprint, log_list::v3::LogList};
use luct_otlsp::OtlspClient;
use luct_scanner::{LogBuilder, Report, Scanner as CtScanner, ScannerConfig, SctReport};
use std::sync::Arc;
use tracing::Level;
use tracing_wasm::WASMLayerConfigBuilder;
use url::Url;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_time::{SystemTime, UNIX_EPOCH};

mod store;

const USER_AGENT: &str = concat!(
    "luct-firefox/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/Sawchord/luct/)"
);

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::default()
            .set_max_level(Level::DEBUG)
            .build(),
    );

    Ok(())
}

#[wasm_bindgen]
pub struct Scanner {
    scanner: CtScanner<RequestDeduplicationClient<OtlspClient>>,
}

#[wasm_bindgen]
impl Scanner {
    #[wasm_bindgen(constructor)]
    pub fn new(log_list: String) -> Result<Self, String> {
        let log_list: LogList = serde_json::from_str(&log_list).map_err(|err| format!("{err}"))?;
        let logs = log_list.currently_active_logs();

        let config = ScannerConfig::builder()
            .build()
            .map_err(|err| format!("{:?}", err))?;

        let client = match config.otlsp_url() {
            Some(url) => OtlspClient::builder()
                .proxy_url(url.clone())
                .connection_timeout(*config.otlsp_connection_timeout())
                .agent(USER_AGENT.to_string())
                .build(),

            None => OtlspClient::builder().agent(USER_AGENT.to_string()).build(),
        };
        let client = RequestDeduplicationClient::new(client);

        let sct_report_cache = Box::new(
            BrowserStore::<[u8; 32], SctReport>::new_local_store("sct_report".to_string())
                .expect("Failed to initialize SCT report cache"),
        ) as _;

        let report_cache = Box::new(
            BrowserStore::<Fingerprint, Report>::new_local_store("report".to_string())
                .expect("Failed to initialize report cache"),
        ) as _;

        let mut scanner =
            CtScanner::new_with_client(config, sct_report_cache, report_cache, client);

        for log in logs {
            let name = log.description();
            scanner.add_log(
                LogBuilder::new(&log).with_sth_store(
                    BrowserStore::new_local_store(format!("sth/{name}"))
                        .expect("Failed to initialize STH store"),
                ),
            );
        }

        log("Initialized scanner");
        Ok(Scanner { scanner })
    }

    #[wasm_bindgen]
    pub async fn collect_report(
        &self,
        url: String,
        certs: Array,
    ) -> Result<Option<JsValue>, String> {
        // Check that this is not a recursion
        if self.is_recursion(&url)? {
            tracing::trace!("Skipping request to log itself to prevent recursion");
            return Ok(None);
        }

        // Parse the certificate
        let cert_chain_bytes = certs
            .to_vec()
            .into_iter()
            .map(|value| Uint8Array::from(value).to_vec())
            .collect::<Vec<_>>();

        let cert_chain =
            CertificateChain::from_der_chain(&cert_chain_bytes).map_err(|err| err.to_string())?;

        // Generate the report
        let report = self
            .scanner
            .collect_report(Arc::new(cert_chain))
            .await
            .map_err(|err| err.to_string())?;

        let report = serde_wasm_bindgen::to_value(&report).map_err(|err| format!("{err}"))?;

        Ok(Some(report))
    }

    #[wasm_bindgen]
    pub fn evaluate_report(report: JsValue) -> Result<(), String> {
        let report: Report =
            serde_wasm_bindgen::from_value(report).map_err(|err| format!("{err}"))?;

        let now = DateTime::from_timestamp_millis(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        )
        .unwrap()
        .into();
        report.evaluate_policy(now)?;

        Ok(())
    }

    /// Check that we are not requesting from a URL that is the log itself
    ///
    /// This is necessary as in the browser, the calls to the logs go through the same
    /// security context and will be intercepted by the browser
    fn is_recursion(&self, url: &str) -> Result<bool, String> {
        let url = Url::parse(url).map_err(|err| format!("{err}"))?;
        let is_recusion = self.scanner.logs().any(|log| {
            log.config().url().domain() == url.domain()
                || log
                    .config()
                    .tile_url()
                    .as_ref()
                    .map(|tile_url| tile_url.domain())
                    == Some(url.domain())
        });

        Ok(is_recusion)
    }
}

// TODO: Full scenario test
