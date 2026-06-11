use std::{
    cell::RefCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use luct_core::store::{
    AppendableStore, AsyncStoreRead, AsyncStoreWrite, OrderedStoreRead, SearchableStoreRead,
    StoreRead, StoreWrite,
};

/// A [`OrderedStore`](luct_core::store::OrderedStore) that caches the `last` value in memory
///
/// If you need to call [`OrderedStoreRead::last`], this will speed up access
pub struct LastValCacheStore<K, V, S> {
    last: RefCell<Option<(K, V)>>,
    inner: S,
    _key: PhantomData<K>,
}

impl<K, V, S> Deref for LastValCacheStore<K, V, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V, S> DerefMut for LastValCacheStore<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K, V, S> LastValCacheStore<K, V, S> {
    pub fn new(store: S) -> Self {
        Self {
            last: RefCell::new(None),
            inner: store,
            _key: PhantomData,
        }
    }
}

impl<K, V, S> StoreRead for LastValCacheStore<K, V, S>
where
    S: StoreRead<Key = K, Value = V>,
{
    type Key = S::Key;
    type Value = S::Value;

    fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V, S> StoreWrite for LastValCacheStore<K, V, S>
where
    S: StoreWrite<Key = K, Value = V>,
{
    fn insert(&self, key: K, value: V) {
        *self.last.borrow_mut() = None;
        self.inner.insert(key, value);
    }

    fn delete(&self, key: &K) -> bool {
        *self.last.borrow_mut() = None;
        self.inner.delete(key)
    }
}

impl<K, V, S> OrderedStoreRead for LastValCacheStore<K, V, S>
where
    K: Ord + Clone,
    V: Clone,
    S: OrderedStoreRead<Key = K, Value = V>,
{
    fn last(&self) -> Option<(K, V)> {
        let mut last_borrow = self.last.borrow_mut();
        match last_borrow.as_ref() {
            Some(last) => Some(last.clone()),
            None => {
                let last = self.inner.last();
                *last_borrow = last.clone();
                last
            }
        }
    }
}

impl<K, V, S> AppendableStore for LastValCacheStore<K, V, S>
where
    K: Ord + Clone,
    V: Clone,
    S: AppendableStore<Key = K, Value = V>,
{
    fn append(&self, value: V) -> K {
        *self.last.borrow_mut() = None;
        self.inner.append(value)
    }
}

impl<K, V, S> SearchableStoreRead for LastValCacheStore<K, V, S>
where
    K: Ord + Clone,
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

impl<K, V, S> AsyncStoreRead for LastValCacheStore<K, V, S>
where
    K: Clone,
    S: AsyncStoreRead<Key = K>,
{
    type Key = <S as AsyncStoreRead>::Key;
    type Value = <S as AsyncStoreRead>::Value;

    async fn get(&self, key: Self::Key) -> Option<Self::Value> {
        self.inner.get(key.clone()).await
    }

    async fn len(&self) -> usize {
        self.inner.len().await
    }
}

impl<K, V, S> AsyncStoreWrite for LastValCacheStore<K, V, S>
where
    K: Clone,
    S: AsyncStoreWrite<Key = K>,
{
    async fn insert(&self, key: Self::Key, value: Self::Value) {
        *self.last.borrow_mut() = None;
        self.inner.insert(key, value).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luct_core::store::MemoryStore;
    use luct_test::store::{ordered_store_test, searchable_store_test, store_test};

    #[test]
    fn last_val_store() {
        let store = LastValCacheStore::new(MemoryStore::<u64, String>::default());
        store_test(store);
    }

    #[test]
    fn last_val_ordered_store() {
        let store = LastValCacheStore::new(MemoryStore::<u64, String>::default());
        ordered_store_test(store);
    }

    #[test]
    fn last_val_searchable_store() {
        let store = LastValCacheStore::new(MemoryStore::<u64, String>::default());
        searchable_store_test(store);
    }
}
