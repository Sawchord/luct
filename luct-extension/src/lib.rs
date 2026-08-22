#![forbid(unsafe_code)]
//! Wrapper around [`Scanner`](CtScanner) to be used in a javascript environment.

use crate::{browser_storage::BrowserStorage, config::load_config};
use chrono::DateTime;
use js_sys::{Array, Uint8Array};
use luct_client::deduplication::RequestDeduplicationClient;
use luct_core::{
    CertificateChain as CertChain, Fingerprint, log_list::v3::LogList, v1::SignedTreeHead,
};
use luct_otlsp::{OtlspClient, OtlspClientConfig};
use luct_scanner::{Report, Scanner as CtScanner, ScannerConfig, ScannerImpl, Validated};
use luct_store::{LruCacheStore, MetadataCacheStore};
use std::sync::Arc;
use tracing::{Level, info};
use tracing_wasm::WASMLayerConfigBuilder;
use url::Url;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_time::{SystemTime, UNIX_EPOCH};

mod browser_storage;
mod config;
mod extension_sys;

const USER_AGENT: &str = concat!(
    "luct-firefox/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/Sawchord/luct/)"
);

struct ExtensionScannerImpl;

impl ScannerImpl for ExtensionScannerImpl {
    type Client = RequestDeduplicationClient<OtlspClient>;
    type ReportStore = LruCacheStore<BrowserStorage<Fingerprint, Report>>;
    type SthStore = MetadataCacheStore<BrowserStorage<u64, Validated<SignedTreeHead>>>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    #[cfg(debug_assertions)]
    let log_level = Level::DEBUG;

    #[cfg(not(debug_assertions))]
    let log_level = Level::DEBUG;

    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::default()
            .set_max_level(log_level)
            .build(),
    );

    Ok(())
}

#[wasm_bindgen]
pub struct CertificateChain {
    cert_chain: CertChain,
}

#[wasm_bindgen]
impl CertificateChain {
    #[wasm_bindgen(constructor)]
    pub fn new(certs: Array) -> Result<Self, String> {
        let cert_chain_bytes = certs
            .to_vec()
            .into_iter()
            .map(|value| Uint8Array::from(value).to_vec())
            .collect::<Vec<_>>();

        let cert_chain =
            CertChain::from_der_chain(&cert_chain_bytes).map_err(|err| err.to_string())?;

        Ok(Self { cert_chain })
    }

    #[wasm_bindgen]
    pub fn report(&self) -> Result<JsValue, String> {
        let report = Report::from(&self.cert_chain);
        let report = serde_wasm_bindgen::to_value(&report).map_err(|err| format!("{err}"))?;

        Ok(report)
    }
}
#[wasm_bindgen]
pub struct Scanner {
    scanner: CtScanner<ExtensionScannerImpl>,
}

#[wasm_bindgen]
impl Scanner {
    #[wasm_bindgen(constructor)]
    pub fn new(log_list: String) -> Result<Self, String> {
        let log_list: LogList = serde_json::from_str(&log_list).map_err(|err| format!("{err}"))?;
        let logs = log_list.currently_active_logs();

        let extension_config = load_config()?;
        let scanner_config = ScannerConfig::try_from(&extension_config)?;
        let otlsp_config = OtlspClientConfig::try_from(&extension_config)?;

        let client = RequestDeduplicationClient::new(OtlspClient::new(otlsp_config));

        let report_cache =
            BrowserStorage::<Fingerprint, Report>::new_local_store("report".to_string())?;
        let report_cache = LruCacheStore::new(report_cache, extension_config.report_lru_cache());

        let time_source = || {
            DateTime::from_timestamp_millis(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64,
            )
            .unwrap()
        };

        let mut scanner = CtScanner::new(scanner_config, report_cache, client, time_source);

        for log in logs {
            let name = log.description();
            scanner.add_log(
                &log,
                MetadataCacheStore::new(BrowserStorage::new_local_store(format!("sth/{name}"))?),
            );
        }

        info!("Initialized scanner");

        Ok(Scanner { scanner })
    }

    #[wasm_bindgen]
    pub async fn collect_report(
        &self,
        url: String,
        certs: CertificateChain,
    ) -> Result<Option<JsValue>, String> {
        // Check that this is not a recursion
        if self.is_recursion(&url)? {
            tracing::trace!("Skipping request to log itself to prevent recursion");
            return Ok(None);
        }

        // Generate the report
        let report = self
            .scanner
            .collect_report(Arc::new(certs.cert_chain))
            .await
            .map_err(|err| err.to_string())?;

        let report = serde_wasm_bindgen::to_value(&report).map_err(|err| format!("{err}"))?;

        Ok(Some(report))
    }

    #[wasm_bindgen]
    pub fn is_report_safe(report: JsValue) -> Result<bool, String> {
        let report: Report =
            serde_wasm_bindgen::from_value(report).map_err(|err| format!("{err}"))?;

        match report.get_error() {
            Some(_) => Ok(false),
            None => Ok(true),
        }
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

    #[wasm_bindgen]
    pub async fn basic_statistics(&self) -> Result<JsValue, String> {
        let stats = self.scanner.basic_statistics().await;
        serde_wasm_bindgen::to_value(&stats).map_err(|err| format!("{err}"))
    }
}

// TODO: Full scenario test
#[cfg(test)]
mod test {
    use super::*;
    use luct_test::utils::test_tracing;
    use serde::{Deserialize, Serialize};
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    // These tests check that the custom deserialization format for
    // `Validated<T>` also work with `serde_wasm_bindgen`
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct TestStruct {
        a: u64,
        b: String,
    }

    #[wasm_bindgen_test]
    fn validated_js_value_roundtrip() {
        test_tracing();

        let test_data = Validated::new(TestStruct {
            a: 5,
            b: String::from("Test"),
        });
        let js = serde_wasm_bindgen::to_value(&test_data).unwrap();
        let new_test_data = serde_wasm_bindgen::from_value(js).unwrap();
        assert_eq!(test_data, new_test_data)
    }

    #[wasm_bindgen_test]
    fn legacy_validated_js_value() {
        test_tracing();

        let test_data = Validated::new(TestStruct {
            a: 5,
            b: String::from("Test"),
        });
        let data_js = serde_wasm_bindgen::to_value(&test_data.inner()).unwrap();
        let now_js = serde_wasm_bindgen::to_value(
            &test_data
                .validated_at()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
        .unwrap();

        let legacy_validated = Array::new();
        legacy_validated.push(&now_js);
        legacy_validated.push(&data_js);

        let new_test_data: Validated<TestStruct> =
            serde_wasm_bindgen::from_value(legacy_validated.into()).unwrap();
        assert_eq!(test_data, new_test_data)
    }
}
