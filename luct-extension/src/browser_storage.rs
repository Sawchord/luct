use futures::lock::Mutex;
use js_sys::{Array, Object, Reflect};
use luct_core::store::{OrderedStoreRead, SearchableStoreRead, StoreBase, StoreRead, StoreWrite};
use luct_store::StringStoreKey;
use serde::{Serialize, de::DeserializeOwned};
use tracing::warn;
use wasm_bindgen::JsValue;

use crate::extension_sys::{StorageArea, browser};
use std::{cmp::Ord, fmt, marker::PhantomData};

#[derive(Debug)]
pub struct BrowserStorage<K, V>(Mutex<BrowserStorageInner<K, V>>);

struct BrowserStorageInner<K, V> {
    _kv: PhantomData<(K, V)>,
    prefix: String,
    storage: StorageArea,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for BrowserStorageInner<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserStorage")
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl<K, V> BrowserStorage<K, V> {
    pub fn new_local_store(prefix: String) -> Result<Self, String> {
        let storage = browser().with(|browser| browser.storage().local());

        Ok(Self(Mutex::new(BrowserStorageInner {
            _kv: PhantomData,
            prefix,
            storage,
        })))
    }
}

impl<K: StringStoreKey, V> BrowserStorageInner<K, V> {
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

    async fn set_item(&self, key: &str, value: &JsValue) {
        let val = Object::new();
        Reflect::set(&val, &JsValue::from(key), value).unwrap();

        self.storage.set(&val).await.expect("Failed to set item");
    }

    async fn get_item(&self, key: &str) -> Option<JsValue> {
        let key = JsValue::from_str(key);
        let request = self
            .storage
            .get(&key)
            .await
            .expect("Failed to call get_item");

        let request = Reflect::get(&request, &key).expect("Failed to get item");

        if request.is_null_or_undefined() {
            None
        } else {
            Some(request)
        }
    }

    async fn remove_item(&self, key: &str) {
        self.storage
            .remove(&key.into())
            .await
            .expect("Failed to remove item");
    }

    async fn get_count(&self) -> usize {
        self.get_item(&self.count_key())
            .await
            .map(|val| val.as_f64().unwrap() as usize)
            .unwrap_or(0)
    }

    async fn inc_count(&self) {
        let count: usize = self.get_count().await;

        self.set_item(&self.count_key(), &JsValue::from(count + 1))
            .await;
    }

    async fn dec_count(&self) {
        let count: usize = self.get_count().await;

        self.set_item(&self.count_key(), &JsValue::from(count - 1))
            .await;
    }
}

impl<K, V> StoreBase for BrowserStorage<K, V> {
    type Key = K;
    type Value = V;
}

impl<K, V> StoreRead for BrowserStorage<K, V>
where
    K: StringStoreKey,
    V: DeserializeOwned,
{
    async fn get(&self, key: Self::Key) -> Option<Self::Value> {
        let value = {
            let storage = self.0.lock().await;
            let key = storage.get_key_string(&key);

            storage.get_item(&key).await?
        };
        let value: Self::Value = serde_wasm_bindgen::from_value(value).unwrap();

        Some(value)
    }

    async fn len(&self) -> usize {
        self.0.lock().await.get_count().await
    }
}

impl<K, V> StoreWrite for BrowserStorage<K, V>
where
    K: StringStoreKey,
    V: Serialize,
{
    async fn insert(&self, key: Self::Key, value: Self::Value) {
        let value = serde_wasm_bindgen::to_value(&value).expect("Failed to convert to JS value");

        let storage = self.0.lock().await;
        let key = storage.get_key_string(&key);

        if storage.get_item(&key).await.is_none() {
            storage.inc_count().await;
        }

        storage.set_item(&key, &value).await;
    }

    async fn delete(&self, key: Self::Key) -> bool {
        let storage = self.0.lock().await;
        let key = storage.get_key_string(&key);

        let had_item = storage.get_item(&key).await.is_some();

        if had_item {
            storage.dec_count().await;
        }

        storage.remove_item(&key).await;
        had_item
    }
}

impl<K, V> OrderedStoreRead for BrowserStorage<K, V>
where
    K: StringStoreKey + Ord,
    V: DeserializeOwned,
{
    async fn last(&self) -> Option<(Self::Key, Self::Value)> {
        // TODO: Remove lock

        let storage = self.0.lock().await;

        let all_keys = storage
            .storage
            .get_keys()
            .await
            .expect("Failed to retrieve keys");

        let mut largest_key = None;
        Array::from(&all_keys).into_iter().for_each(|key| {
            let key_str = key.as_string().unwrap();
            let Some(key) = storage.key_from_str(&key_str) else {
                return;
            };

            match &largest_key {
                None => largest_key = Some(key),
                Some(old_key) => {
                    if old_key < &key {
                        largest_key = Some(key)
                    }
                }
            }
        });

        let largest_key = largest_key?;
        let largest_key_str = storage.get_key_string(&largest_key);

        let val = storage.get_item(&largest_key_str).await.unwrap();
        let val: Self::Value =
            serde_wasm_bindgen::from_value(val).expect("Failed to deserialize a stored value");

        Some((largest_key, val))
    }
}

impl<K, V> SearchableStoreRead for BrowserStorage<K, V>
where
    K: StringStoreKey + Ord,
    V: DeserializeOwned,
{
    async fn filter(
        &self,
        mut pred: impl FnMut(&Self::Key, &Self::Value) -> bool,
    ) -> Vec<(Self::Key, Self::Value)> {
        // TODO: Move prefix outside of lock and drop lock before iterating through elements

        let storage = self.0.lock().await;

        let all_elems = storage
            .storage
            .get(&JsValue::null())
            .await
            .expect("Failed to retrieve all values");

        let mut matches = vec![];
        let mut errors = 0;
        Object::entries(&Object::from(all_elems)).for_each(&mut |elem, _, _| {
            let elem = Array::from(&elem);

            let key_str = elem.get(0).as_string().unwrap();
            let Some(key) = storage.key_from_str(&key_str) else {
                return;
            };

            let value: Self::Value = match serde_wasm_bindgen::from_value(elem.get(1)) {
                Ok(value) => value,
                Err(_) => {
                    errors += 1;
                    return;
                }
            };

            if pred(&key, &value) {
                matches.push((key, value));
            }
        });

        if errors != 0 {
            warn!("{} elements could not be deserialized", errors)
        }

        matches
    }

    // TODO: Efficient find implementation
}
