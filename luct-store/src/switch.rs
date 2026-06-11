use luct_core::store::{
    AppendableStore, OrderedStoreRead, SearchableStoreRead, StoreBase, StoreRead, StoreWrite,
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

impl<A, B, K, V> AppendableStore for StoreSwitch<A, B>
where
    K: Ord,
    A: AppendableStore<Key = K, Value = V>,
    B: AppendableStore<Key = K, Value = V>,
{
    fn append(&self, value: V) -> K {
        match self {
            StoreSwitch::A(a) => a.append(value),
            StoreSwitch::B(b) => b.append(value),
        }
    }
}

impl<A, B, K, V> SearchableStoreRead for StoreSwitch<A, B>
where
    K: Ord,
    A: SearchableStoreRead<Key = K, Value = V>,
    B: SearchableStoreRead<Key = K, Value = V>,
{
    fn filter(&self, pred: impl FnMut(&K, &V) -> bool) -> Vec<(K, V)> {
        match self {
            StoreSwitch::A(a) => a.filter(pred),
            StoreSwitch::B(b) => b.filter(pred),
        }
    }

    fn find(&self, pred: impl FnMut(&K, &V) -> bool) -> Option<(K, V)> {
        match self {
            StoreSwitch::A(a) => a.find(pred),
            StoreSwitch::B(b) => b.find(pred),
        }
    }
}
