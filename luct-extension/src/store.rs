use js_sys::Object;
use luct_core::store::{OrderedStoreRead, SearchableStoreRead, StoreRead, StoreWrite};
use luct_store::{StringStoreKey, StringStoreValue};
use std::marker::PhantomData;
use web_sys::{Storage, window};

#[derive(Debug)]
pub struct BrowserStore<K, V> {
    _kv: PhantomData<(K, V)>,
    prefix: String,
    storage: Storage,
}

impl<K, V> BrowserStore<K, V> {
    pub fn new_local_store(prefix: String) -> Option<Self> {
        let storage = window().map(|window| window.local_storage())?.ok()??;
        Some(Self {
            _kv: PhantomData,
            prefix,
            storage,
        })
    }
}

impl<K: StringStoreKey, V> BrowserStore<K, V> {
    fn get_key_string(&self, key: &K) -> String {
        format!("{}/{}", self.prefix, key.serialize_key())
    }

    fn key_from_str(&self, key: &str) -> Option<K> {
        if !key.starts_with(&self.prefix) || key.chars().nth(self.prefix.len()) != Some('/') {
            return None;
        }

        K::deserialize_key(&key[self.prefix.len() + 1..])
    }

    fn count_key(&self) -> String {
        format!("{}#count", self.prefix)
    }

    fn get_count(&self) -> usize {
        self.storage
            .get_item(&self.count_key())
            .expect("Failed to retrieve count")
            .unwrap_or("0".to_string())
            .parse()
            .expect("Count contains non integer value")
    }

    fn inc_count(&self) {
        let count: usize = self.get_count();

        self.storage
            .set_item(&self.count_key(), &(count + 1).to_string())
            .expect("Failed to set count");
    }

    fn dec_count(&self) {
        let count: usize = self.get_count();

        self.storage
            .set_item(&self.count_key(), &(count - 1).to_string())
            .expect("Failed to set count");
    }
}

impl<K: StringStoreKey, V: StringStoreValue> StoreRead<K, V> for BrowserStore<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        let key = self.get_key_string(key);

        self.storage
            .get_item(&key)
            .expect("Failed to retreive value from local store")
            .and_then(|val| V::deserialize_value(&val))
    }

    fn len(&self) -> usize {
        self.get_count()
    }
}

impl<K: StringStoreKey, V: StringStoreValue> StoreWrite<K, V> for BrowserStore<K, V> {
    fn insert(&self, key: K, value: V) {
        let key = self.get_key_string(&key);
        let val = value.serialize_value();

        if self
            .storage
            .get_item(&key)
            .expect("Failed to retreive value into local store")
            .is_none()
        {
            self.inc_count();
        }

        self.storage
            .set_item(&key, &val)
            .expect("Failed to insert value into local store");
    }

    fn delete(&self, key: &K) -> bool {
        let key = self.get_key_string(key);
        let had_item = self
            .storage
            .get_item(&key)
            .expect("Failed to retreive value from local store")
            .is_some();

        if had_item {
            self.dec_count();
        }

        self.storage
            .remove_item(&key)
            .expect("Failed to remove value from local store");

        had_item
    }
}

impl<K: StringStoreKey + Ord, V: StringStoreValue> OrderedStoreRead<K, V> for BrowserStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        let key = Object::keys(&self.storage)
            .iter()
            .filter_map(|key| key.as_string())
            .filter_map(|key| self.key_from_str(&key))
            .max()?;

        let key_str = self.get_key_string(&key);
        let val = self
            .storage
            .get_item(&key_str)
            .expect("Failed to retreive last element of store")?;
        let val = V::deserialize_value(&val)?;

        Some((key, val))
    }
}

impl<K: StringStoreKey + Ord, V: StringStoreValue> SearchableStoreRead<K, V>
    for BrowserStore<K, V>
{
    fn filter(&self, mut pred: impl FnMut(&K, &V) -> bool) -> Vec<(K, V)> {
        Object::keys(&self.storage)
            .iter()
            .filter_map(|key| key.as_string())
            .filter_map(|key| self.key_from_str(&key))
            .filter_map(|key| {
                self.storage
                    .get_item(&self.get_key_string(&key))
                    .expect("Failed to reteive element from store")
                    .map(|data| (key, data))
            })
            .filter_map(|(key, data)| V::deserialize_value(&data).map(|val| (key, val)))
            .filter(|(key, val)| pred(key, val))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use luct_test::store::{ordered_store_test, searchable_store_test, store_test};
    use tracing::Level;
    use tracing_subscriber::{Registry, layer::SubscriberExt};
    use tracing_wasm::{ConsoleConfig, WASMLayer, WASMLayerConfigBuilder};
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn browser_store() {
        clear_storage();
        tracing();

        let store = BrowserStore::new_local_store("test".to_string()).unwrap();
        store_test(store);
    }

    #[wasm_bindgen_test]
    fn browser_ordered_store() {
        clear_storage();
        tracing();

        let store = BrowserStore::new_local_store("test".to_string()).unwrap();
        ordered_store_test(store);
    }

    #[wasm_bindgen_test]
    fn browser_searchable_store() {
        clear_storage();
        tracing();

        let store = BrowserStore::new_local_store("test".to_string()).unwrap();
        searchable_store_test(store);
    }

    // Clears the storage before starting a test
    fn clear_storage() {
        window()
            .unwrap()
            .local_storage()
            .unwrap()
            .unwrap()
            .clear()
            .unwrap();
    }

    fn tracing() {
        let _ = tracing::subscriber::set_global_default(
            Registry::default().with(WASMLayer::new(
                WASMLayerConfigBuilder::default()
                    .set_max_level(Level::TRACE)
                    .set_console_config(ConsoleConfig::ReportWithoutConsoleColor)
                    .build(),
            )),
        );
    }
}
