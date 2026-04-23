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

    // Insert a smaller element check that largest element remains unchanged
    assert_eq!(store.get(&3), None);
    store.insert(3, "three".to_string());
    assert_eq!(store.len(), 3);
    assert_eq!(store.get(&3), Some("three".to_string()));
    assert_eq!(store.last(), Some((4, "four".to_string())));

    // Remove a smaller element and check that the larger element remains unchanged
    assert!(store.delete(&3));
    assert_eq!(store.len(), 2);
    assert_eq!(store.get(&3), None);
    assert!(!store.delete(&3));
    assert_eq!(store.last(), Some((4, "four".to_string())));

    // Remove the largest element and check that a smaller element is now the largest
    assert!(store.delete(&4));
    assert_eq!(store.len(), 1);
    assert_eq!(store.get(&4), None);
    assert!(!store.delete(&4));
    assert_eq!(store.last(), Some((2, "two".to_string())));
}

// TODO: Iterator store test
// -- Insert elements out of order
// -- Check that iteration works in correct order
// -- Remove some elements
// -- Check that order is presereved

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

    #[test]
    fn memory_ordered_store() {
        let store = MemoryStore::<u64, String>::default();
        ordered_store_test(store);
    }
}
