use crate::store::{
    AsyncOrderedStoreRead, AsyncSearchableStoreRead, AsyncStoreRead, AsyncStoreWrite,
    OrderedStoreRead, SearchableStoreRead, StoreBase, StoreRead, StoreWrite,
};

pub struct AsyncAdapter<S>(S);

impl<S> AsyncAdapter<S> {
    pub fn new(store: S) -> Self {
        Self(store)
    }
}

impl<S: StoreBase> StoreBase for AsyncAdapter<S> {
    type Key = S::Key;
    type Value = S::Value;
}

impl<S: StoreRead> AsyncStoreRead for AsyncAdapter<S> {
    async fn get(&self, key: Self::Key) -> Option<Self::Value> {
        self.0.get(&key)
    }

    async fn len(&self) -> usize {
        self.0.len()
    }
}

impl<S: StoreWrite> AsyncStoreWrite for AsyncAdapter<S> {
    async fn insert(&self, key: Self::Key, value: Self::Value) {
        self.0.insert(key, value);
    }

    async fn delete(&self, key: Self::Key) -> bool {
        self.0.delete(&key)
    }
}

impl<S: OrderedStoreRead> AsyncOrderedStoreRead for AsyncAdapter<S> {
    async fn last(&self) -> Option<(Self::Key, Self::Value)> {
        self.0.last()
    }
}

impl<S: SearchableStoreRead> AsyncSearchableStoreRead for AsyncAdapter<S> {
    async fn filter(
        &self,
        pred: impl FnMut(&Self::Key, &Self::Value) -> bool,
    ) -> Vec<(Self::Key, Self::Value)> {
        self.0.filter(pred)
    }

    async fn find<'a>(
        &'a self,
        pred: impl FnMut(&Self::Key, &Self::Value) -> bool + 'a,
    ) -> Option<(Self::Key, Self::Value)> {
        self.0.find(pred)
    }
}
