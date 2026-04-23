use crate::tree::HashOutput;

mod r#async;
mod memory;

pub use crate::store::r#async::{AsyncStore, AsyncStoreRead, AsyncStoreWrite};
pub use crate::store::memory::MemoryStore;

/// Trait indicating that an object can be hased with respect to the CT protocol
///
/// This for now always refers to the Sha256 algorithm, but this might change in the future
pub trait Hashable {
    /// Hash the object
    fn hash(&self) -> HashOutput;
}

pub trait StoreRead<K, V> {
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

pub trait StoreWrite<K, V> {
    /// Insert a value into the store
    ///
    /// # Arguments:
    /// - `key`: the key associated with the value
    /// - `value`: the value itself
    fn insert(&self, key: K, value: V);

    /// Remove a value from the store
    ///
    /// # Arguments
    /// - `key`: the key to be removed
    ///
    /// # Returns
    /// - `true` if the key existed and has been removed
    /// - `false` otherwise
    fn delete(&self, key: &K) -> bool;
}

/// The [`Store`] trait is a basic key-value store trait
///
/// Note that there is no ACID requirement in the trait.
pub trait Store<K, V>: StoreRead<K, V> + StoreWrite<K, V> {}
impl<K, V, T> Store<K, V> for T where T: StoreRead<K, V> + StoreWrite<K, V> {}

/// Extension to regular [`Stores`](Store), which have ordered keys
pub trait OrderedStoreRead<K: Ord, V>: StoreRead<K, V> {
    /// Returns the last element in the store
    ///
    /// The last element is the largest element with respect to the keys [`Ord`] implementation.
    ///
    /// # Returns
    /// - `Some(key, value)` if the store is non-empty
    /// - `None` otherwise
    fn last(&self) -> Option<(K, V)>;
}

pub trait OrderedStore<K: Ord, V>: OrderedStoreRead<K, V> + StoreWrite<K, V> {}

impl<K, V, T> OrderedStore<K, V> for T
where
    K: Ord,
    T: OrderedStoreRead<K, V> + StoreWrite<K, V>,
{
}

/// Extension to regular [`Stores`](Store), which use an index as a key
///
/// The main difference is, that the key is a [`u64`] and the store determines the key when inserting
pub trait AppendableStore<K: Ord, V>: OrderedStoreRead<K, V> {
    /// Insert a value into the store and return the index
    ///
    /// # Arguments:
    /// - `value`: the value itself
    ///
    /// # Returns:
    /// - the index of the new value. This is the key under which the value can later be retreived
    fn append(&self, value: V) -> K;
}

/// Extension to a [`OrderedStoreRead`], that allows looking through the store to look for specific
/// entries,
pub trait SearchableStoreRead<K: Ord, V>: OrderedStoreRead<K, V> {
    /// Search for all entries in the store, that fulfill a certain predicate
    ///
    /// Note that the elements are being searched through in the order specified by [`Ord`] of key
    ///
    /// # Arguments
    /// - `pred`: A predicate that has access to the key and value
    ///
    /// # Returns
    /// - An array of key-value pairs, for which `pred` holds true
    fn filter<F: FnMut(&K, &V) -> bool>(&self, pred: F) -> Vec<(K, V)>;

    fn find<F: FnMut(&K, &V) -> bool>(&self, mut pred: F) -> Option<(K, V)> {
        let mut found = false;

        let vals = self.filter(|key, value| {
            if !found && pred(key, value) {
                found = true;
                true
            } else {
                false
            }
        });

        if found {
            assert_eq!(vals.len(), 1);
            Some(vals.into_iter().next().unwrap())
        } else {
            None
        }
    }
}

pub trait SearchableStore<K: Ord, V>: SearchableStoreRead<K, V> + StoreWrite<K, V> {}

impl<K, V, T> SearchableStore<K, V> for T
where
    K: Ord,
    T: SearchableStoreRead<K, V> + StoreWrite<K, V>,
{
}
