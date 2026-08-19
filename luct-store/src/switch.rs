use luct_core::store::{
    AsyncAppendableStore, AsyncOrderedStoreRead, AsyncSearchableStoreRead, AsyncStoreRead,
    AsyncStoreWrite, OrderedStoreRead, StoreBase, StoreRead, StoreWrite,
};

/// [`Store`](luct_core::store::Store) implementation that switches between two different
/// inner [`Stores`](luct_core::store::Store).
///
/// This can be used, if you want to switch at runtime between the [`Stores`](luct_core::store::Store)
/// [`A`](Self::A) and [`B`](Self::B),
pub enum StoreSwitch<A, B> {
    A(A),
    B(B),
}

impl<A, B, K, V> StoreBase for StoreSwitch<A, B>
where
    A: StoreBase<Key = K, Value = V>,
    B: StoreBase<Key = K, Value = V>,
{
    type Key = K;
    type Value = V;
}

impl<A, B, K, V> StoreRead for StoreSwitch<A, B>
where
    A: StoreRead<Key = K, Value = V>,
    B: StoreRead<Key = K, Value = V>,
{
    fn get(&self, key: &K) -> Option<V> {
        match self {
            StoreSwitch::A(a) => a.get(key),
            StoreSwitch::B(b) => b.get(key),
        }
    }

    fn len(&self) -> usize {
        match self {
            StoreSwitch::A(a) => a.len(),
            StoreSwitch::B(b) => b.len(),
        }
    }
}

impl<A, B, K, V> StoreWrite for StoreSwitch<A, B>
where
    A: StoreWrite<Key = K, Value = V>,
    B: StoreWrite<Key = K, Value = V>,
{
    fn insert(&self, key: K, value: V) {
        match self {
            StoreSwitch::A(a) => a.insert(key, value),
            StoreSwitch::B(b) => b.insert(key, value),
        }
    }

    fn delete(&self, key: &K) -> bool {
        match self {
            StoreSwitch::A(a) => a.delete(key),
            StoreSwitch::B(b) => b.delete(key),
        }
    }
}

impl<A, B, K, V> OrderedStoreRead for StoreSwitch<A, B>
where
    K: Ord,
    A: OrderedStoreRead<Key = K, Value = V>,
    B: OrderedStoreRead<Key = K, Value = V>,
{
    fn last(&self) -> Option<(K, V)> {
        match self {
            StoreSwitch::A(a) => a.last(),
            StoreSwitch::B(b) => b.last(),
        }
    }
}

impl<A, B, K, V> AsyncStoreRead for StoreSwitch<A, B>
where
    A: AsyncStoreRead<Key = K, Value = V>,
    B: AsyncStoreRead<Key = K, Value = V>,
{
    async fn get(&self, key: K) -> Option<V> {
        match self {
            StoreSwitch::A(a) => a.get(key).await,
            StoreSwitch::B(b) => b.get(key).await,
        }
    }

    async fn len(&self) -> usize {
        match self {
            StoreSwitch::A(a) => a.len().await,
            StoreSwitch::B(b) => b.len().await,
        }
    }
}

impl<A, B, K, V> AsyncStoreWrite for StoreSwitch<A, B>
where
    A: AsyncStoreWrite<Key = K, Value = V>,
    B: AsyncStoreWrite<Key = K, Value = V>,
{
    async fn insert(&self, key: K, value: V) {
        match self {
            StoreSwitch::A(a) => a.insert(key, value).await,
            StoreSwitch::B(b) => b.insert(key, value).await,
        }
    }

    async fn delete(&self, key: K) -> bool {
        match self {
            StoreSwitch::A(a) => a.delete(key).await,
            StoreSwitch::B(b) => b.delete(key).await,
        }
    }
}

impl<A, B, K, V> AsyncOrderedStoreRead for StoreSwitch<A, B>
where
    K: Ord,
    A: AsyncOrderedStoreRead<Key = K, Value = V>,
    B: AsyncOrderedStoreRead<Key = K, Value = V>,
{
    async fn last(&self) -> Option<(K, V)> {
        match self {
            StoreSwitch::A(a) => a.last().await,
            StoreSwitch::B(b) => b.last().await,
        }
    }
}

impl<A, B, K, V> AsyncAppendableStore for StoreSwitch<A, B>
where
    K: Ord,
    A: AsyncAppendableStore<Key = K, Value = V>,
    B: AsyncAppendableStore<Key = K, Value = V>,
{
    async fn append(&self, value: V) -> K {
        match self {
            StoreSwitch::A(a) => a.append(value).await,
            StoreSwitch::B(b) => b.append(value).await,
        }
    }
}

impl<A, B, K, V> AsyncSearchableStoreRead for StoreSwitch<A, B>
where
    K: Ord,
    A: AsyncSearchableStoreRead<Key = K, Value = V>,
    B: AsyncSearchableStoreRead<Key = K, Value = V>,
{
    async fn filter(&self, pred: impl FnMut(&K, &V) -> bool) -> Vec<(K, V)> {
        match self {
            StoreSwitch::A(a) => a.filter(pred).await,
            StoreSwitch::B(b) => b.filter(pred).await,
        }
    }

    async fn find(&self, pred: impl FnMut(&K, &V) -> bool) -> Option<(K, V)> {
        match self {
            StoreSwitch::A(a) => a.find(pred).await,
            StoreSwitch::B(b) => b.find(pred).await,
        }
    }
}
