use luct_core::store::{
    AppendableStore, OrderedStoreRead, SearchableStoreRead, StoreBase, StoreRead, StoreWrite,
};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
struct CachedMetadata<S>
where
    S: StoreBase,
{
    last: Option<(S::Key, S::Value)>,
    len: Option<usize>,
}

impl<S> Default for CachedMetadata<S>
where
    S: StoreBase,
{
    fn default() -> Self {
        Self {
            last: None,
            len: None,
        }
    }
}

/// A [`OrderedStore`](luct_core::store::OrderedStore) that caches some metadata values in memeoty
///
/// Some methods such as [`StoreRead::len`] or [`OrderedStoreRead::last`] might be slow
/// to call on some [`Store`](luct_core::store::Store) implementations.
///
/// This wrapper will cache the results returned by these calls and return the
/// same value on successive calls, until a write call is made.
pub struct MetadataCacheStore<S>
where
    S: StoreBase,
{
    meta: RefCell<CachedMetadata<S>>,
    inner: S,
}

impl<S> Deref for MetadataCacheStore<S>
where
    S: StoreBase,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> DerefMut for MetadataCacheStore<S>
where
    S: StoreBase,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S> MetadataCacheStore<S>
where
    S: StoreBase,
{
    pub fn new(store: S) -> Self {
        Self {
            meta: RefCell::new(CachedMetadata::default()),
            inner: store,
        }
    }

    fn reset_metadata(&self) {
        *self.meta.borrow_mut() = CachedMetadata::default();
    }
}

impl<S> StoreBase for MetadataCacheStore<S>
where
    S: StoreBase,
{
    type Key = S::Key;
    type Value = S::Value;
}

impl<S> StoreRead for MetadataCacheStore<S>
where
    S: StoreRead<Key: Clone>,
{
    async fn get(&self, key: Self::Key) -> Option<Self::Value> {
        self.inner.get(key.clone()).await
    }

    async fn len(&self) -> usize {
        let len = self.meta.borrow().len;

        if let Some(len) = len {
            len
        } else {
            let new_len = self.inner.len().await;
            self.meta.borrow_mut().len = Some(new_len);
            new_len
        }
    }
}

impl<S> StoreWrite for MetadataCacheStore<S>
where
    S: StoreWrite<Key: Clone>,
{
    async fn insert(&self, key: Self::Key, value: Self::Value) {
        self.reset_metadata();
        self.inner.insert(key, value).await
    }

    async fn delete(&self, key: Self::Key) -> bool {
        self.reset_metadata();
        self.inner.delete(key).await
    }
}

impl<S> OrderedStoreRead for MetadataCacheStore<S>
where
    S: OrderedStoreRead<Key: Clone, Value: Clone>,
{
    async fn last(&self) -> Option<(Self::Key, Self::Value)> {
        let last = self.meta.borrow().last.clone();

        if let Some(last) = last {
            Some(last)
        } else {
            let new_last = self.inner.last().await;
            self.meta.borrow_mut().last = new_last.clone();
            new_last
        }
    }
}

impl<S> AppendableStore for MetadataCacheStore<S>
where
    S: AppendableStore<Key: Clone, Value: Clone>,
{
    async fn append(&self, value: Self::Value) -> Self::Key {
        self.reset_metadata();
        self.inner.append(value).await
    }
}

impl<S> SearchableStoreRead for MetadataCacheStore<S>
where
    S: SearchableStoreRead<Key: Clone, Value: Clone>,
{
    async fn filter(
        &self,
        pred: impl FnMut(&Self::Key, &Self::Value) -> bool,
    ) -> Vec<(Self::Key, Self::Value)> {
        self.inner.filter(pred).await
    }

    async fn find(
        &self,
        pred: impl FnMut(&Self::Key, &Self::Value) -> bool,
    ) -> Option<(Self::Key, Self::Value)> {
        self.inner.find(pred).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luct_core::store::MemoryStore;
    use luct_test::store::{ordered_store_test, searchable_store_test, store_test};

    #[tokio::test]
    async fn metadata_cache_store() {
        let store = MetadataCacheStore::new(MemoryStore::<u64, String>::default());
        store_test(store).await;
    }

    #[tokio::test]
    async fn metadata_cache_ordered_store() {
        let store = MetadataCacheStore::new(MemoryStore::<u64, String>::default());
        ordered_store_test(store).await;
    }

    #[tokio::test]
    async fn metadata_cache_searchable_store() {
        let store = MetadataCacheStore::new(MemoryStore::<u64, String>::default());
        searchable_store_test(store).await;
    }
}
