use crate::tree::HashOutput;

mod r#async;
mod memory;

pub use crate::store::r#async::{
    AsyncAppendableStore, AsyncOrderedStore, AsyncOrderedStoreRead, AsyncSearchableStore,
    AsyncSearchableStoreRead, AsyncStore, AsyncStoreRead, AsyncStoreWrite,
};
pub use crate::store::memory::MemoryStore;

/// Trait indicating that an object can be hased with respect to the CT protocol
///
/// This for now always refers to the Sha256 algorithm, but this might change in the future
pub trait Hashable {
    /// Hash the object
    fn hash(&self) -> HashOutput;
}

pub trait StoreBase {
    type Key;
    type Value;
}

pub trait StoreRead: StoreBase {
    /// Returns the value associated with `key` from the [`Store`]
    ///
    /// # Arguments:
    /// - `key`: the key indexing the object
    ///
    /// # Returns:
    /// - `Some(value)`, if the value exists
    /// - `None` otherwise
    fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    /// Returns the number of elements in the [`Store`]
    fn len(&self) -> usize;

    /// Returns `true`, if the store is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait StoreWrite: StoreBase {
    /// Insert a value into the store
    ///
    /// # Arguments:
    /// - `key`: the key associated with the value
    /// - `value`: the value itself
    fn insert(&self, key: Self::Key, value: Self::Value);

    /// Remove a value from the store
    ///
    /// # Arguments
    /// - `key`: the key to be removed
    ///
    /// # Returns
    /// - `true` if the key existed and has been removed
    /// - `false` otherwise
    fn delete(&self, key: &Self::Key) -> bool;
}

/// The [`Store`] trait is a basic key-value store trait
///
/// Note that there is no ACID requirement in the trait.
pub trait Store: StoreRead + StoreWrite {}
impl<T> Store for T where T: StoreRead + StoreWrite {}

/// Extension to regular [`Stores`](Store), which have ordered keys
pub trait OrderedStoreRead: StoreRead<Key: Ord> {
    /// Returns the last element in the store
    ///
    /// The last element is the largest element with respect to the keys [`Ord`] implementation.
    ///
    /// # Returns
    /// - `Some(key, value)` if the store is non-empty
    /// - `None` otherwise
    fn last(&self) -> Option<(Self::Key, Self::Value)>;
}

pub trait OrderedStore: OrderedStoreRead + StoreWrite {}
impl<T> OrderedStore for T where T: OrderedStoreRead + StoreWrite {}

/// Extension to regular [`Stores`](Store), which use an index as a key
///
/// The main difference is, that the values can be inserted without providing a key.
/// The key is then returned after insertion.
///
/// The key that was returned last must have be the largest value wrt [`Ord`].
pub trait AppendableStore: OrderedStoreRead {
    /// Insert a value into the store and return the index
    ///
    /// # Arguments:
    /// - `value`: the value itself
    ///
    /// # Returns:
    /// - the index of the new value. This is the key under which the value can later be retreived
    fn append(&self, value: Self::Value) -> Self::Key;
}
