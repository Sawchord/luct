use luct_core::store::Store;

/// Basic store test that tests abillity to store and retreive items
pub fn store_test<S: Store<Key = u64, Value = String>>(store: S) {
    assert!(store.is_empty());

    // Check that store persists values
    assert_eq!(store.get(&2), None);
    store.insert(2, "two".to_string());
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(&2), Some("two".to_string()));

    // Insert second element
    assert_eq!(store.get(&1), None);
    store.insert(1, "one".to_string());
    assert_eq!(store.len(), 2);
    assert_eq!(store.get(&1), Some("one".to_string()));

    // Overwrite an element
    store.insert(2, "no longer two".to_string());
    assert_eq!(store.len(), 2);
    assert_eq!(store.get(&2), Some("no longer two".to_string()));

    // Test that deleting works properly
    assert!(store.delete(&2));
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(&2), None);
    assert_eq!(store.get(&1), Some("one".to_string()));

    // Test that you can rewrite to a previously deleted element
    store.insert(2, "it was two once".to_string());
    assert_eq!(store.len(), 2);
    assert_eq!(store.get(&2), Some("it was two once".to_string()));
}

// TODO: Multistore test?

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
