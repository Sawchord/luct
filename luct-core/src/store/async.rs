use crate::store::StoreBase;
use std::future::Future;

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

pub trait AsyncStoreWrite: AsyncStoreRead {
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

/// The [`AsyncStore`] trait is a version of the [`Store`](crate::store::Store) that is asynchrounous
///
/// This allows the underlying store engine to make asynchronous requests,
/// such as a distributed storage or rebuilding the store dynamically using tiles
pub trait AsyncStore: AsyncStoreRead + AsyncStoreWrite {}

impl<T> AsyncStore for T where T: AsyncStoreRead + AsyncStoreWrite {}

/// Async version of [`OrderedStore`](crate::store::OrderedStore)
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

/// Async version of [`AppendableStore`](crate::store::AppendableStore)
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

/// Async version of [`SearchableStore`](crate::store::SearchableStore)
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
        pred: impl FnMut(Self::Key, Self::Value) -> bool,
    ) -> impl Future<Output = Vec<(Self::Key, Self::Value)>>;

    fn find<'a>(
        &'a self,
        mut pred: impl FnMut(Self::Key, Self::Value) -> bool + 'a,
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
