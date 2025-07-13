use crate::tree::HashOutput;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

pub trait Hashable {
    fn hash(&self) -> HashOutput;
}

pub trait Store<K, V> {
    fn insert(&self, key: K, value: V);
    fn get(&self, key: &K) -> Option<V>;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait OrderedStore<K: Ord, V>: Store<K, V> {
    fn last(&self) -> Option<V>;
}

pub trait IndexedStore<V>: Store<u64, V> {
    fn insert_indexed(&self, value: V) -> u64;
}

#[derive(Debug, Clone)]
pub struct MemoryStore<K, V>(Arc<RwLock<BTreeMap<K, V>>>);

impl<K: Ord, V> Default for MemoryStore<K, V> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(BTreeMap::new())))
    }
}

impl<K: Ord, V: Clone> Store<K, V> for MemoryStore<K, V> {
    fn insert(&self, key: K, value: V) {
        self.0.write().unwrap().insert(key, value);
    }

    fn get(&self, key: &K) -> Option<V> {
        self.0.read().unwrap().get(key).cloned()
    }

    fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }
}

impl<K: Ord, V: Clone> OrderedStore<K, V> for MemoryStore<K, V> {
    fn last(&self) -> Option<V> {
        self.0
            .read()
            .unwrap()
            .iter()
            .next_back()
            .map(|(_, v)| v.clone())
    }
}

impl<V: Clone> IndexedStore<V> for MemoryStore<u64, V> {
    fn insert_indexed(&self, value: V) -> u64 {
        let mut store = self.0.write().unwrap();

        let len = store.len() as u64;
        store.insert(len, value);

        len
    }
}
