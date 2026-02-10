use luct_core::store::{OrderedStore, Store};
use luct_store::{StringStoreKey, StringStoreValue};
use std::marker::PhantomData;
use web_sys::{Storage, window};

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

    fn last_key(&self) -> String {
        format!("{}#last", self.prefix)
    }

    fn set_last(&self, key: &str) {
        self.storage
            .set_item(&self.last_key(), key)
            .expect("Failed to set last value")
    }
}

impl<K: StringStoreKey, V: StringStoreValue> Store<K, V> for BrowserStore<K, V> {
    fn insert(&self, key: K, value: V) {
        let key = self.get_key_string(&key);
        let val = value.serialize_value();

        match self
            .storage
            .get_item(&key)
            .expect("Failed to retrive value from local store")
        {
            Some(old_val) if old_val == val => return,
            Some(old_val) => panic!(
                "Tried to overwrite value {} with {}. This is a bug",
                old_val, val
            ),
            None => (),
        };

        self.storage
            .set_item(&key, &val)
            .expect("Failed to insert value into local store");

        self.inc_count();
        self.set_last(&key);
    }

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

impl<K: StringStoreKey + Ord, V: StringStoreValue> OrderedStore<K, V> for BrowserStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        let key = self
            .storage
            .get_item(&self.last_key())
            .expect("Failed to retrieve last key")?;
        let val = self
            .storage
            .get_item(&key)
            .expect("Failed to retreive last element of store")?;

        let key = self.key_from_str(&key)?;
        let val = V::deserialize_value(&val)?;

        Some((key, val))
    }
}

// TODO: Unit tests for browser local store
