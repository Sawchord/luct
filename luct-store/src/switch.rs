use luct_core::store::{
    AppendableStore, OrderedStoreRead, SearchableStoreRead, StoreRead, StoreWrite,
};

pub enum StoreSwitch<A, B> {
    A(A),
    B(B),
}

impl<A, B, K, V> StoreRead<K, V> for StoreSwitch<A, B>
where
    A: StoreRead<K, V>,
    B: StoreRead<K, V>,
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

impl<A, B, K, V> StoreWrite<K, V> for StoreSwitch<A, B>
where
    A: StoreWrite<K, V>,
    B: StoreWrite<K, V>,
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

impl<A, B, K, V> OrderedStoreRead<K, V> for StoreSwitch<A, B>
where
    K: Ord,
    A: OrderedStoreRead<K, V>,
    B: OrderedStoreRead<K, V>,
{
    fn last(&self) -> Option<(K, V)> {
        match self {
            StoreSwitch::A(a) => a.last(),
            StoreSwitch::B(b) => b.last(),
        }
    }
}

impl<A, B, K, V> AppendableStore<K, V> for StoreSwitch<A, B>
where
    K: Ord,
    A: AppendableStore<K, V>,
    B: AppendableStore<K, V>,
{
    fn append(&self, value: V) -> K {
        match self {
            StoreSwitch::A(a) => a.append(value),
            StoreSwitch::B(b) => b.append(value),
        }
    }
}

impl<A, B, K, V> SearchableStoreRead<K, V> for StoreSwitch<A, B>
where
    K: Ord,
    A: SearchableStoreRead<K, V>,
    B: SearchableStoreRead<K, V>,
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
