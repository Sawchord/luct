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
pub struct LastValCache<K, V, S> {
    last: RefCell<Option<(K, V)>>,
    inner: S,
    _key: PhantomData<K>,
}

impl<K, V, S> Deref for LastValCache<K, V, S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V, S> DerefMut for LastValCache<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K, V, S> LastValCache<K, V, S> {
    pub fn new(store: S) -> Self {
        Self {
            last: RefCell::new(None),
            inner: store,
            _key: PhantomData,
        }
    }
}

impl<K, V, S> StoreRead<K, V> for LastValCache<K, V, S>
where
    S: StoreRead<K, V>,
{
    fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V, S> StoreWrite<K, V> for LastValCache<K, V, S>
where
    S: StoreWrite<K, V>,
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

impl<K, V, S> OrderedStoreRead<K, V> for LastValCache<K, V, S>
where
    K: Ord + Clone,
    V: Clone,
    S: OrderedStoreRead<K, V>,
{
    fn last(&self) -> Option<(K, V)> {
        match self.last.borrow().as_ref() {
            Some(last) => Some(last.clone()),
            None => {
                let last = self.inner.last();
                *self.last.borrow_mut() = last.clone();
                last
            }
        }
    }
}

impl<K, V, S> AppendableStore<K, V> for LastValCache<K, V, S>
where
    K: Ord + Clone,
    V: Clone,
    S: AppendableStore<K, V>,
{
    fn append(&self, value: V) -> K {
        *self.last.borrow_mut() = None;
        self.inner.append(value)
    }
}

impl<K, V, S> SearchableStoreRead<K, V> for LastValCache<K, V, S>
where
    K: Ord + Clone,
    V: Clone,
    S: SearchableStoreRead<K, V>,
{
    fn filter(&self, pred: impl FnMut(&K, &V) -> bool) -> Vec<(K, V)> {
        self.inner.filter(pred)
    }

    fn find(&self, pred: impl FnMut(&K, &V) -> bool) -> Option<(K, V)> {
        self.inner.find(pred)
    }
}

impl<K, V, S> AsyncStoreRead<K, V> for LastValCache<K, V, S>
where
    K: Clone,
    S: AsyncStoreRead<K, V>,
{
    async fn get(&self, key: K) -> Option<V> {
        self.inner.get(key.clone()).await
    }

    async fn len(&self) -> usize {
        self.inner.len().await
    }
}

impl<K, V, S> AsyncStoreWrite<K, V> for LastValCache<K, V, S>
where
    S: AsyncStoreWrite<K, V>,
{
    async fn insert(&self, key: K, value: V) {
        *self.last.borrow_mut() = None;
        self.inner.insert(key, value).await
    }
}
