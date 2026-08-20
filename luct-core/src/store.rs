pub use crate::store::memory::MemoryStore;
use crate::tree::HashOutput;
use std::future::Future;

mod memory;

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

pub trait AsyncStoreRead: StoreBase {
    /// Returns the value associated with `key` from the [`Store`](crate::store::Store)
    ///
    /// # Arguments:
    /// - `key`: the key indexing the object
    ///
    /// # Returns:
    /// - `Some(value)`, if the value exists
    /// - `None` otherwise
    fn get(&self, key: Self::Key) -> impl Future<Output = Option<Self::Value>>;

    /// Returns the number of elements in the [`Store`](crate::store::Store)
    fn len(&self) -> impl Future<Output = usize>;

    /// Returns `true`, if the store is empty
    fn is_empty(&self) -> impl Future<Output = bool> {
        async { self.len().await == 0 }
    }
}

pub trait AsyncStoreWrite: StoreBase {
    /// Insert a value into the store
    ///
    /// # Arguments:
    /// - `key`: the key associated with the value
    /// - `value`: the value itself
    fn insert(&self, key: Self::Key, value: Self::Value) -> impl Future<Output = ()>;

    /// Remove a value from the store
    ///
    /// # Arguments
    /// - `key`: the key to be removed
    ///
    /// # Returns
    /// - `true` if the key existed and has been removed
    /// - `false` otherwise
    fn delete(&self, key: Self::Key) -> impl Future<Output = bool>;
}

/// The [`Store`] trait is a basic key-value store trait
///
/// Note that there is no ACID requirement in the trait.
pub trait AsyncStore: AsyncStoreRead + AsyncStoreWrite {}

impl<T> AsyncStore for T where T: AsyncStoreRead + AsyncStoreWrite {}

/// Extension to regular [`Stores`](Store), which have ordered keys
pub trait AsyncOrderedStoreRead: AsyncStoreRead<Key: Ord> {
    /// Returns the last element in the store
    ///
    /// The last element is the largest element with respect to the keys [`Ord`] implementation.
    ///
    /// # Returns
    /// - `Some(key, value)` if the store is non-empty
    /// - `None` otherwise
    fn last(&self) -> impl Future<Output = Option<(Self::Key, Self::Value)>>;
}

pub trait AsyncOrderedStore: AsyncOrderedStoreRead + AsyncStoreWrite {}
impl<T> AsyncOrderedStore for T where T: AsyncOrderedStoreRead + AsyncStoreWrite {}

/// Extension to regular [`Stores`](Store), which use an index as a key
///
/// The main difference is, that the values can be inserted without providing a key.
/// The key is then returned after insertion.
///
/// The key that was returned last must have be the largest value wrt [`Ord`].
pub trait AsyncAppendableStore: AsyncOrderedStoreRead {
    /// Insert a value into the store and return the index
    ///
    /// # Arguments:
    /// - `value`: the value itself
    ///
    /// # Returns:
    /// - the index of the new value. This is the key under which the value can later be retreived
    fn append(&self, value: Self::Value) -> impl Future<Output = Self::Key>;
}

/// Extension to a [`OrderedStoreRead`], that allows looking through the store to look for specific
/// entries.
pub trait AsyncSearchableStoreRead: AsyncOrderedStoreRead {
    /// Search for all entries in the store, that fulfill a certain predicate
    ///
    /// Note that the elements are being searched through in the order specified by [`Ord`] of key
    ///
    /// # Arguments
    /// - `pred`: A predicate that has access to the key and value
    ///
    /// # Returns
    /// - An array of key-value pairs, for which `pred` holds true
    fn filter(
        &self,
        pred: impl FnMut(&Self::Key, &Self::Value) -> bool,
    ) -> impl Future<Output = Vec<(Self::Key, Self::Value)>>;

    fn find(
        &self,
        mut pred: impl FnMut(&Self::Key, &Self::Value) -> bool,
    ) -> impl Future<Output = Option<(Self::Key, Self::Value)>> {
        async move {
            let mut found = false;

            let vals = self
                .filter(|key, value| {
                    if !found && pred(key, value) {
                        found = true;
                        true
                    } else {
                        false
                    }
                })
                .await;

            if found {
                assert_eq!(vals.len(), 1);
                Some(vals.into_iter().next().unwrap())
            } else {
                None
            }
        }
    }
}

pub trait AsyncSearchableStore: AsyncSearchableStoreRead + AsyncStoreWrite {}
impl<T> AsyncSearchableStore for T where T: AsyncSearchableStoreRead + AsyncStoreWrite {}
