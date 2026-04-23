use luct_core::store::{OrderedStore, Store};

/// Basic store test that tests abillity to store and retreive items
pub fn store_test<S: Store<u64, String>>(store: S) {
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

    // Overwrite an elelement
    store.insert(2, "no longer two".to_string());
    assert_eq!(store.len(), 2);
    assert_eq!(store.get(&2), Some("no longer two".to_string()));

    // TODO: Store test
    // -- Delete element 2 check that it no longer exists and 1 does
}

/// Tests capabilities of an ordered store
pub fn ordered_store_test<S: OrderedStore<u64, String>>(store: S) {
    assert!(store.is_empty());

    // Insert an element
    assert_eq!(store.get(&2), None);
    store.insert(2, "two".to_string());
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(&2), Some("two".to_string()));
    assert_eq!(store.last(), Some((2, "two".to_string())));

    // Insert a larger element, check that is now last
    assert_eq!(store.get(&4), None);
    store.insert(4, "four".to_string());
    assert_eq!(store.len(), 2);
    assert_eq!(store.get(&4), Some("four".to_string()));
    assert_eq!(store.last(), Some((4, "four".to_string())));

    // Insert a smaller element check that largest element remains unchainged
    assert_eq!(store.get(&3), None);
    store.insert(3, "three".to_string());
    assert_eq!(store.len(), 3);
    assert_eq!(store.get(&3), Some("three".to_string()));
    assert_eq!(store.last(), Some((4, "four".to_string())));

    // TODO: Ordered store test
    // -- Delete three check that four remains largest elelemtn
    // -- Delete four check that two is largest element
}

// TODO: Iterator store test
// -- Insert elements out of order
// -- Check that iteration works in correct order
// -- Remove some elements
// -- Check that order is presereved

#[cfg(test)]
mod tests {
    use super::*;
    use luct_core::store::MemoryStore;

    #[test]
    fn memory_store() {
        let store = MemoryStore::<u64, String>::default();
        store_test(store);
    }

    #[test]
    fn memory_ordered_store() {
        let store = MemoryStore::<u64, String>::default();
        ordered_store_test(store);
    }
}
