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
