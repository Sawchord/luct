use js_sys::{Array, Object, Reflect};
use luct_core::store::{OrderedStoreRead, SearchableStoreRead, StoreBase, StoreRead, StoreWrite};
use luct_store::StringStoreKey;
use serde::{Serialize, de::DeserializeOwned};
use tracing::warn;
use wasm_bindgen::JsValue;

use crate::extension_sys::{StorageArea, browser};
use std::{cmp::Ord, fmt, marker::PhantomData};

pub struct BrowserStorage<K, V> {
    _kv: PhantomData<(K, V)>,
    prefix: String,
    storage: StorageArea,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for BrowserStorage<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserStorage")
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl<K, V> BrowserStorage<K, V> {
    pub fn new_local_store(prefix: String) -> Result<Self, String> {
        let storage = browser().with(|browser| browser.storage().local());

        Ok(Self {
            _kv: PhantomData,
            prefix,
            storage,
        })
    }
}

impl<K: StringStoreKey, V> BrowserStorage<K, V> {
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
            let key = self.get_key_string(&key);
            self.get_item(&key).await?
        };
        let value: Self::Value = serde_wasm_bindgen::from_value(value).unwrap();

        Some(value)
    }

    async fn len(&self) -> usize {
        let all_keys = self.storage.get_keys().await.expect("Failed to get keys");

        let keys = Array::from(&all_keys).filter(&mut |key: JsValue, _, _| {
            let key_str = key.as_string().unwrap();
            self.key_from_str(&key_str).is_some()
        });

        keys.length() as usize
    }
}

impl<K, V> StoreWrite for BrowserStorage<K, V>
where
    K: StringStoreKey,
    V: Serialize,
{
    async fn insert(&self, key: Self::Key, value: Self::Value) {
        let value = serde_wasm_bindgen::to_value(&value).expect("Failed to convert to JS value");

        // Migration code to remove the old count values
        // TODO: Remove after rollout of 0.3.0
        self.remove_item(&self.count_key()).await;

        let key = self.get_key_string(&key);
        self.set_item(&key, &value).await;
    }

    async fn delete(&self, key: Self::Key) -> bool {
        let key = self.get_key_string(&key);
        let had_item = self.get_item(&key).await.is_some();

        self.remove_item(&key).await;
        had_item
    }
}

impl<K, V> OrderedStoreRead for BrowserStorage<K, V>
where
    K: StringStoreKey + Ord,
    V: DeserializeOwned,
{
    async fn last(&self) -> Option<(Self::Key, Self::Value)> {
        let all_keys = self
            .storage
            .get_keys()
            .await
            .expect("Failed to retrieve keys");

        let mut largest_key = None;
        Array::from(&all_keys).into_iter().for_each(|key| {
            let key_str = key.as_string().unwrap();
            let Some(key) = self.key_from_str(&key_str) else {
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
        let largest_key_str = self.get_key_string(&largest_key);

        let val = self.get_item(&largest_key_str).await.unwrap();
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
        let all_elems = self
            .storage
            .get(&JsValue::null())
            .await
            .expect("Failed to retrieve all values");

        let mut matches = vec![];
        let mut errors = 0;
        Object::entries(&Object::from(all_elems)).for_each(&mut |elem, _, _| {
            let elem = Array::from(&elem);

            let key_str = elem.get(0).as_string().unwrap();
            let Some(key) = self.key_from_str(&key_str) else {
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
