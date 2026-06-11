use lru::LruCache;
use luct_core::store::{
    AppendableStore, AsyncStoreRead, AsyncStoreWrite, OrderedStoreRead, SearchableStoreRead, Store,
    StoreBase, StoreRead, StoreWrite,
};
use std::{
    cell::RefCell,
    fmt::Debug,
    hash::Hash,
    ops::{Deref, DerefMut},
};

/// A [`Store`](luct_core::store::Store) implementation that wraps an inner [`Store`](luct_core::store::Store)
/// and ads an LRU (least-recently-used) cache around it.
///
/// The cache is write-through, i.e. there is no speedup when writing to the store.
/// Furthermore, the implementation is not [`Send`] or [`Sync`].
/// A common patthern would be to have one [`LruCacheStore`] per thread wrapping an inner store.
pub struct LruCacheStore<S>
where
    S: StoreBase,
{
    cache: RefCell<LruCache<S::Key, S::Value>>,
    inner: S,
}

impl<S> Debug for LruCacheStore<S>
where
    S: StoreBase<Key: Debug, Value: Debug> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LruCacheStore")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<S> Deref for LruCacheStore<S>
where
    S: StoreBase,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> DerefMut for LruCacheStore<S>
where
    S: Store,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S> LruCacheStore<S>
where
    S: StoreBase<Key: Hash + Eq>,
{
    pub fn new(store: S, caps: usize) -> Self {
        Self {
            cache: RefCell::new(LruCache::new(caps.try_into().unwrap())),
            inner: store,
        }
    }
}

impl<S> StoreBase for LruCacheStore<S>
where
    S: StoreBase,
{
    type Key = S::Key;
    type Value = S::Value;
}

impl<S> StoreRead for LruCacheStore<S>
where
    S: StoreRead<Key: Clone + Hash + Eq, Value: Clone>,
{
    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        if let Some(val) = self.cache.borrow_mut().get(key) {
            Some(val.clone())
        } else {
            let val = self.inner.get(key)?;
            self.cache.borrow_mut().put(key.clone(), val.clone());
            Some(val)
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<S> StoreWrite for LruCacheStore<S>
where
    S: StoreWrite<Key: Hash + Eq>,
{
    fn insert(&self, key: Self::Key, value: Self::Value) {
        self.cache.borrow_mut().pop(&key);
        self.inner.insert(key, value);
    }

    fn delete(&self, key: &Self::Key) -> bool {
        let contained = self.inner.delete(key);
        self.cache.borrow_mut().pop(key);
        contained
    }
}

impl<S> OrderedStoreRead for LruCacheStore<S>
where
    S: OrderedStoreRead<Key: Clone + Hash, Value: Clone>,
{
    fn last(&self) -> Option<(Self::Key, Self::Value)> {
        self.inner.last()
    }
}

impl<S> AppendableStore for LruCacheStore<S>
where
    S: AppendableStore<Key: Clone + Hash, Value: Clone>,
{
    fn append(&self, value: Self::Value) -> Self::Key {
        self.inner.append(value)
    }
}

impl<S> SearchableStoreRead for LruCacheStore<S>
where
    S: SearchableStoreRead<Key: Clone + Hash, Value: Clone>,
{
    fn filter(
        &self,
        pred: impl FnMut(&Self::Key, &Self::Value) -> bool,
    ) -> Vec<(Self::Key, Self::Value)> {
        self.inner.filter(pred)
    }

    fn find(
        &self,
        pred: impl FnMut(&Self::Key, &Self::Value) -> bool,
    ) -> Option<(Self::Key, Self::Value)> {
        self.inner.find(pred)
    }
}

impl<S> AsyncStoreRead for LruCacheStore<S>
where
    S: AsyncStoreRead<Key: Clone + Hash + Eq, Value: Clone>,
{
    async fn get(&self, key: Self::Key) -> Option<Self::Value> {
        if let Some(val) = self.cache.borrow_mut().get(&key) {
            Some(val.clone())
        } else {
            let val = self.inner.get(key.clone()).await?;
            self.cache.borrow_mut().put(key, val.clone());
            Some(val)
        }
    }

    async fn len(&self) -> usize {
        self.inner.len().await
    }
}

impl<S> AsyncStoreWrite for LruCacheStore<S>
where
    S: AsyncStoreWrite<Key: Clone + Hash + Eq, Value: Clone>,
{
    async fn insert(&self, key: Self::Key, value: Self::Value) {
        self.cache.borrow_mut().pop(&key);
        self.inner.insert(key, value).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luct_core::store::MemoryStore;
    use luct_test::store::{ordered_store_test, searchable_store_test, store_test};

    #[test]
    fn lru_cache_store() {
        let store = LruCacheStore::new(MemoryStore::<u64, String>::default(), 1000);
        store_test(store);
    }

    #[test]
    fn lru_cache_ordered_store() {
        let store = LruCacheStore::new(MemoryStore::<u64, String>::default(), 1000);
        ordered_store_test(store);
    }

    #[test]
    fn lru_cache_searchable_store() {
        let store = LruCacheStore::new(MemoryStore::<u64, String>::default(), 1000);
        searchable_store_test(store);
    }
}
