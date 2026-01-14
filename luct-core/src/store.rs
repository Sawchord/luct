use crate::tree::HashOutput;
use std::{
    collections::BTreeMap,
    future::Future,
    sync::{Arc, RwLock},
};

/// Trait indicating that an object can be hased with respect to the CT protocol
///
/// This for now always refers to the Sha256 algorithm, but this might change in the future
pub trait Hashable {
    /// Hash the object
    fn hash(&self) -> HashOutput;
}

/// The [`Store`] trait is a basic key-value store trait
///
/// Note that there is no ACID requirement in the trait.
/// In fact, there is no notion of deletion whatsoever.
pub trait Store<K, V> {
    /// Insert a value into the store
    ///
    /// # Arguments:
    /// - `key`: the key associated with the value
    /// - `value`: the value itself
    fn insert(&self, key: K, value: V);

    /// Returns the value associated with `key` from the [`Store`]
    ///
    /// # Arguments:
    /// - `key`: the key indexing the object
    ///
    /// # Returns:
    /// - `Some(value)`, if the value exists
    /// - `None` otherwise
    fn get(&self, key: &K) -> Option<V>;

    /// Returns the number of elements in the [`Store`]
    fn len(&self) -> usize;

    /// Returns `true`, if the store is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Extension to regular [`Stores`](Store), which have ordered keys
pub trait OrderedStore<K: Ord, V>: Store<K, V> {
    /// Returns the last element in the store
    ///
    /// The last element is the largest element with respect to the keys [`Ord`] implementation.
    ///
    /// # Returns
    /// - `Some(key, value)` if the store is non-empty
    /// - `None` otherwise
    fn last(&self) -> Option<(K, V)>;
}

/// Extension to regular [`Stores`](Store), which use an index as a key
///
/// The main difference is, that the key is a [`u64`] and the store determines the key when inserting
pub trait IndexedStore<V>: Store<u64, V> {
    /// Insert a value into the store and return the index
    ///
    /// # Arguments:
    /// - `value`: the value itself
    ///
    /// # Returns:
    /// - the index of the new value. This is the key under which the value can later be retreived
    fn insert_indexed(&self, value: V) -> u64;
}

/// The [`AsyncStore`] trait is a version of the [`Store`] that is asynchrounous
///
/// This allows the underlying store engine to make asynchronous requests,
/// such as a distributed storage or rebuilding the store dynamically using tiles
pub trait AsyncStore<K, V> {
    /// Insert a value into the store
    ///
    /// # Arguments:
    /// - `key`: the key associated with the value
    /// - `value`: the value itself
    fn insert(&self, key: K, value: V) -> impl Future<Output = ()>;

    /// Returns the value associated with `key` from the [`Store`]
    ///
    /// # Arguments:
    /// - `key`: the key indexing the object
    ///
    /// # Returns:
    /// - `Some(value)`, if the value exists
    /// - `None` otherwise
    fn get(&self, key: &K) -> impl Future<Output = Option<V>>;

    /// Returns the number of elements in the [`Store`]
    fn len(&self) -> impl Future<Output = usize>;

    /// Returns `true`, if the store is empty
    fn is_empty(&self) -> impl Future<Output = bool> {
        async { self.len().await == 0 }
    }
}

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

impl<K: Ord + Clone, V: Clone> OrderedStore<K, V> for MemoryStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        self.0
            .read()
            .unwrap()
            .iter()
            .next_back()
            .map(|(k, v)| (k.clone(), v.clone()))
    }
}

impl<V: Clone> IndexedStore<V> for MemoryStore<u64, V> {
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

impl<K: Ord, V: Clone> AsyncStore<K, V> for MemoryStore<K, V> {
    async fn insert(&self, key: K, value: V) {
        self.0.write().unwrap().insert(key, value);
    }

    async fn get(&self, key: &K) -> Option<V> {
        self.0.read().unwrap().get(key).cloned()
    }

    async fn len(&self) -> usize {
        self.0.read().unwrap().len()
    }
}
