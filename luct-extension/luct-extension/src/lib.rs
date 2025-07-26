//! Wrapper around [`Scanner`](CtScanner) to be used in a javascript environment.

use std::collections::BTreeMap;

use js_sys::Array;
use luct_client::reqwest::ReqwestClient;
use luct_core::{CtLogConfig, store::MemoryStore, v1::SignedTreeHead};
use luct_scanner::Scanner as CtScanner;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Called when the Wasm module is instantiated
#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_inner_html("Hello from Rust!");

    body.append_child(&val)?;

    Ok(())
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

        Ok(Scanner(scanner))
    }

    #[wasm_bindgen]
    pub async fn collect_leads(&self, _leads: Array) {
        todo!()
    }
}
