use crate::store::StoreBase;

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
}

/// The [`AsyncStore`] trait is a version of the [`Store`](crate::store::Store) that is asynchrounous
///
/// This allows the underlying store engine to make asynchronous requests,
/// such as a distributed storage or rebuilding the store dynamically using tiles
pub trait AsyncStore: AsyncStoreRead + AsyncStoreWrite {}

impl<T> AsyncStore for T where T: AsyncStoreRead + AsyncStoreWrite {}
