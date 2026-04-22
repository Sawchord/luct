use luct_core::store::Store;

/// Basic store test that tests abillity to store and retreive items
pub fn store_test<S: Store<u64, String>>(store: S) {
    assert!(store.is_empty());

    // Check that store persists values
    assert_eq!(store.get(&2), None);
    store.insert(2, "two".to_string());
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(&2), Some("two".to_string()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use luct_core::store::MemoryStore;

    #[test]
    fn memory_store() {
        let store = MemoryStore::<u64, String>::default();
        store_test(store);
    }
}
