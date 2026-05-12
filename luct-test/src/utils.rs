#[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
use tracing_subscriber::layer::SubscriberExt;

pub fn test_tracing() {
    #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt()
            .compact()
            .with_env_filter(env_filter)
            .init();
    }

    #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))]
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::Registry::default().with(tracing_wasm::WASMLayer::new(
            tracing_wasm::WASMLayerConfigBuilder::default()
                .set_max_level(tracing::Level::TRACE)
                .set_console_config(tracing_wasm::ConsoleConfig::ReportWithoutConsoleColor)
                .build(),
        )),
    );
}
