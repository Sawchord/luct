use lru::LruCache;
use luct_core::store::{
    AppendableStore, AsyncStoreRead, AsyncStoreWrite, OrderedStoreRead, SearchableStoreRead,
    StoreRead, StoreWrite,
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
pub struct LruCacheStore<K, V, S> {
    cache: RefCell<LruCache<K, V>>,
    inner: S,
}

impl<K, V, S> Debug for LruCacheStore<K, V, S>
where
    K: Debug + Hash + Eq,
    V: Debug,
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LruCacheStore")
            .field("cache", &self.cache)
            .field("inner", &self.inner)
            .finish()
    }
}

impl<K, V, S> Deref for LruCacheStore<K, V, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V, S> DerefMut for LruCacheStore<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K: Hash + Eq, V, S> LruCacheStore<K, V, S> {
    pub fn new(store: S, caps: usize) -> Self {
        Self {
            cache: RefCell::new(LruCache::new(caps.try_into().unwrap())),
            inner: store,
        }
    }
}

impl<K, V, S> StoreRead for LruCacheStore<K, V, S>
where
    K: Clone + Hash + Eq,
    V: Clone,
    S: StoreRead<Key = K, Value = V>,
{
    type Key = S::Key;
    type Value = S::Value;

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

impl<K, V, S> StoreWrite for LruCacheStore<K, V, S>
where
    K: Clone + Hash + Eq,
    V: Clone,
    S: StoreWrite<Key = K, Value = V>,
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

impl<K, V, S> OrderedStoreRead for LruCacheStore<K, V, S>
where
    K: Clone + Hash + Eq + Ord,
    V: Clone,
    S: OrderedStoreRead<Key = K, Value = V>,
{
    fn last(&self) -> Option<(Self::Key, Self::Value)> {
        self.inner.last()
    }
}

impl<K, V, S> AppendableStore for LruCacheStore<K, V, S>
where
    K: Clone + Hash + Eq + Ord,
    V: Clone,
    S: AppendableStore<Key = K, Value = V>,
{
    fn append(&self, value: V) -> K {
        self.inner.append(value)
    }
}

impl<K, V, S> SearchableStoreRead for LruCacheStore<K, V, S>
where
    K: Clone + Hash + Eq + Ord,
    V: Clone,
    S: SearchableStoreRead<Key = K, Value = V>,
{
    fn filter(&self, pred: impl FnMut(&K, &V) -> bool) -> Vec<(K, V)> {
        self.inner.filter(pred)
    }

    fn find(&self, pred: impl FnMut(&K, &V) -> bool) -> Option<(K, V)> {
        self.inner.find(pred)
    }
}

impl<K, V, S> AsyncStoreRead for LruCacheStore<K, V, S>
where
    K: Clone + Hash + Eq,
    V: Clone,
    S: AsyncStoreRead<Key = K, Value = V>,
{
    type Key = <S as AsyncStoreRead>::Key;
    type Value = <S as AsyncStoreRead>::Value;

    async fn get(&self, key: K) -> Option<Self::Value> {
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

impl<K, V, S> AsyncStoreWrite for LruCacheStore<K, V, S>
where
    K: Clone + Hash + Eq,
    V: Clone,
    S: AsyncStoreWrite<Key = K, Value = V>,
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
