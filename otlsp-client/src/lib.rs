mod client;
mod error;
mod ws_stream;
use wasm_bindgen::prelude::wasm_bindgen;

pub use client::OtlspClientBuilder;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub(crate) fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (crate::log(&format_args!($($t)*).to_string()))
}
pub(crate) use console_log;
