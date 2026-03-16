use luct_core::Fingerprint;
use luct_scanner::Report;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_time::{Duration, Instant};

/// Every time we serialize a report, we initalize a new [`JsValue`].
///
/// Therefore, caching on the wasm side and then deserializing may create a large amount of identical values.
///
/// Instead, we serialize the values only once and then cache them, and a cache hit then leads to a shallow copy
#[derive(Debug)]
pub(crate) struct ReportCache {
    time_to_idle: Duration,
    last_gc: Instant,
    cache: HashMap<Fingerprint, (Instant, JsValue)>,
}

impl ReportCache {
    pub(crate) fn new(time_to_idle: Duration) -> Self {
        Self {
            time_to_idle,
            last_gc: Instant::now(),
            cache: HashMap::new(),
        }
    }

    pub(crate) fn get(&mut self, fingerprint: &Fingerprint) -> Option<JsValue> {
        let (last_access, value) = self.cache.get_mut(fingerprint)?;
        *last_access = Instant::now();
        let value = value.clone();

        self.try_gc();
        Some(value)
    }

    pub(crate) fn insert(
        &mut self,
        fingerprint: Fingerprint,
        report: &Report,
    ) -> Result<JsValue, String> {
        let report = serde_wasm_bindgen::to_value(&report).map_err(|err| format!("{err}"))?;
        self.cache
            .insert(fingerprint, (Instant::now(), report.clone()));

        self.try_gc();
        Ok(report)
    }

    fn try_gc(&mut self) {
        if self.last_gc + self.time_to_idle < Instant::now() {
            self.gc();
        }
    }

    fn gc(&mut self) {
        let now = Instant::now();

        let new_cache = self
            .cache
            .drain()
            .filter(|(_, (last_access, _))| *last_access + self.time_to_idle > now)
            .collect();

        self.cache = new_cache;
        self.last_gc = now;
    }
}
