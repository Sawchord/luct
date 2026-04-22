use crate::store::{
    AsyncStoreRead, AsyncStoreWrite, IndexedStoreRead, OrderedStoreRead, StoreRead, StoreWrite,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

/// A non-persistent implementation of [`Store`]
///
/// This can be useful for testing, or in settings, in which the data should not be stored,
/// for example when running as a command line tool in CI
#[derive(Debug, Clone)]
pub struct MemoryStore<K, V>(Arc<RwLock<BTreeMap<K, V>>>);

impl<K: Ord, V> Default for MemoryStore<K, V> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(BTreeMap::new())))
    }
}

impl<K: Ord, V: Clone> StoreRead<K, V> for MemoryStore<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        self.0.read().unwrap().get(key).cloned()
    }

    fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }
}

impl<K: Ord, V: Clone> StoreWrite<K, V> for MemoryStore<K, V> {
    fn insert(&self, key: K, value: V) {
        self.0.write().unwrap().insert(key, value);
    }
}

impl<K: Ord + Clone, V: Clone> OrderedStoreRead<K, V> for MemoryStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        self.0
            .read()
            .unwrap()
            .iter()
            .next_back()
            .map(|(k, v)| (k.clone(), v.clone()))
    }
}

impl<V: Clone> IndexedStoreRead<V> for MemoryStore<u64, V> {
    fn insert_indexed(&self, value: V) -> u64 {
        let mut store = self.0.write().unwrap();

        let len = store.len() as u64;
        let old = store.insert(len, value);

        assert!(
            old.is_none(),
            "IndexedStore already contained a value at {len}"
        );

        len
    }
}

impl<K: Ord, V: Clone> AsyncStoreRead<K, V> for MemoryStore<K, V> {
    async fn get(&self, key: K) -> Option<V> {
        self.0.read().unwrap().get(&key).cloned()
    }

    async fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }
}

impl<K: Ord, V: Clone> AsyncStoreWrite<K, V> for MemoryStore<K, V> {
    async fn insert(&self, key: K, value: V) {
        self.0.write().unwrap().insert(key, value);
    }
}
