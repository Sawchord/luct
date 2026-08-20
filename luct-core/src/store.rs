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
