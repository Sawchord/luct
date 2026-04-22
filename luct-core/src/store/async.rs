pub trait AsyncStoreRead<K, V> {
    /// Returns the value associated with `key` from the [`Store`]
    ///
    /// # Arguments:
    /// - `key`: the key indexing the object
    ///
    /// # Returns:
    /// - `Some(value)`, if the value exists
    /// - `None` otherwise
    fn get(&self, key: K) -> impl Future<Output = Option<V>>;

    /// Returns the number of elements in the [`Store`]
    fn len(&self) -> impl Future<Output = usize>;

    /// Returns `true`, if the store is empty
    fn is_empty(&self) -> impl Future<Output = bool> {
        async { self.len().await == 0 }
    }
}

pub trait AsyncStoreWrite<K, V> {
    /// Insert a value into the store
    ///
    /// # Arguments:
    /// - `key`: the key associated with the value
    /// - `value`: the value itself
    fn insert(&self, key: K, value: V) -> impl Future<Output = ()>;
}

/// The [`AsyncStore`] trait is a version of the [`Store`] that is asynchrounous
///
/// This allows the underlying store engine to make asynchronous requests,
/// such as a distributed storage or rebuilding the store dynamically using tiles
pub trait AsyncStore<K, V>: AsyncStoreRead<K, V> + AsyncStoreWrite<K, V> {}

impl<K, V, T> AsyncStore<K, V> for T where T: AsyncStoreRead<K, V> + AsyncStoreWrite<K, V> {}
