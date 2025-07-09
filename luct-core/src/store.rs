use crate::tree::HashOutput;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

pub trait Hashable {
    fn hash(&self) -> HashOutput;
}

pub trait Store<K: Ord, V> {
    fn insert(&self, key: K, value: V) -> Option<V>;
    fn get(&self, key: &K) -> Option<V>;
    fn len(&self) -> usize;
    fn last(&self) -> Option<V>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStore<K, V>(Arc<RwLock<BTreeMap<K, V>>>);

impl<K: Ord, V> Default for MemoryStore<K, V> {
    fn default() -> Self {
        Self(Arc::new(RwLock::new(BTreeMap::new())))
    }
}

impl<K: Ord, V: Clone> Store<K, V> for MemoryStore<K, V> {
    fn insert(&self, key: K, value: V) -> Option<V> {
        self.0.write().unwrap().insert(key, value).clone()
    }

    fn get(&self, key: &K) -> Option<V> {
        self.0.read().unwrap().get(key).cloned()
    }

    fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }

    fn last(&self) -> Option<V> {
        self.0
            .read()
            .unwrap()
            .iter()
            .next_back()
            .map(|(_, v)| v.clone())
    }
}
