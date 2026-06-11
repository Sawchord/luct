use luct_core::store::{
    AppendableStore, AsyncStoreRead, AsyncStoreWrite, OrderedStoreRead, SearchableStoreRead,
    StoreBase, StoreRead, StoreWrite,
};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

/// A [`OrderedStore`](luct_core::store::OrderedStore) that caches the `last` value in memory
///
/// If you need to call [`OrderedStoreRead::last`], this will speed up access
pub struct LastValCacheStore<S>
where
    S: StoreBase,
{
    last: RefCell<Option<(S::Key, S::Value)>>,
    inner: S,
}

impl<S> Deref for LastValCacheStore<S>
where
    S: StoreBase,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> DerefMut for LastValCacheStore<S>
where
    S: StoreBase,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S> LastValCacheStore<S>
where
    S: StoreBase,
{
    pub fn new(store: S) -> Self {
        Self {
            last: RefCell::new(None),
            inner: store,
        }
    }
}

impl<S> StoreBase for LastValCacheStore<S>
where
    S: StoreBase,
{
    type Key = S::Key;
    type Value = S::Value;
}

impl<S> StoreRead for LastValCacheStore<S>
where
    S: StoreRead,
{
    fn get(&self, key: &Self::Key) -> Option<Self::Value> {
        self.inner.get(key)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<S> StoreWrite for LastValCacheStore<S>
where
    S: StoreWrite,
{
    fn insert(&self, key: Self::Key, value: Self::Value) {
        *self.last.borrow_mut() = None;
        self.inner.insert(key, value);
    }

    fn delete(&self, key: &Self::Key) -> bool {
        *self.last.borrow_mut() = None;
        self.inner.delete(key)
    }
}

impl<S> OrderedStoreRead for LastValCacheStore<S>
where
    S: OrderedStoreRead<Key: Clone, Value: Clone>,
{
    fn last(&self) -> Option<(Self::Key, Self::Value)> {
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

impl<S> AppendableStore for LastValCacheStore<S>
where
    S: AppendableStore<Key: Clone, Value: Clone>,
{
    fn append(&self, value: Self::Value) -> Self::Key {
        *self.last.borrow_mut() = None;
        self.inner.append(value)
    }
}

impl<S> SearchableStoreRead for LastValCacheStore<S>
where
    S: SearchableStoreRead<Key: Clone, Value: Clone>,
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

impl<S> AsyncStoreRead for LastValCacheStore<S>
where
    S: AsyncStoreRead<Key: Clone>,
{
    async fn get(&self, key: Self::Key) -> Option<Self::Value> {
        self.inner.get(key.clone()).await
    }

    async fn len(&self) -> usize {
        self.inner.len().await
    }
}

impl<S> AsyncStoreWrite for LastValCacheStore<S>
where
    S: AsyncStoreWrite<Key: Clone>,
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
