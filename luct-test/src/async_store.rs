use luct_core::store::{AsyncOrderedStore, AsyncSearchableStore, AsyncStore};

/// Basic store test that tests abillity to store and retreive items
pub async fn async_store_test<S: AsyncStore<Key = u64, Value = String>>(store: S) {
    assert!(store.is_empty().await);

    // Check that store persists values
    assert_eq!(store.get(2).await, None);
    store.insert(2, "two".to_string()).await;
    assert_eq!(store.len().await, 1);
    assert_eq!(store.get(2).await, Some("two".to_string()));

    // Insert second element
    assert_eq!(store.get(1).await, None);
    store.insert(1, "one".to_string()).await;
    assert_eq!(store.len().await, 2);
    assert_eq!(store.get(1).await, Some("one".to_string()));

    // Overwrite an element
    store.insert(2, "no longer two".to_string()).await;
    assert_eq!(store.len().await, 2);
    assert_eq!(store.get(2).await, Some("no longer two".to_string()));

    // Test that deleting works properly
    assert!(store.delete(2).await);
    assert_eq!(store.len().await, 1);
    assert_eq!(store.get(2).await, None);
    assert_eq!(store.get(1).await, Some("one".to_string()));

    // Test that you can rewrite to a previously deleted element
    store.insert(2, "it was two once".to_string()).await;
    assert_eq!(store.len().await, 2);
    assert_eq!(store.get(2).await, Some("it was two once".to_string()));
}

/// Tests capabilities of an ordered store
pub async fn async_ordered_store_test<S: AsyncOrderedStore<Key = u64, Value = String>>(store: S) {
    assert!(store.is_empty().await);

    // Insert an element
    assert_eq!(store.get(2).await, None);
    store.insert(2, "two".to_string()).await;
    assert_eq!(store.len().await, 1);
    assert_eq!(store.get(2).await, Some("two".to_string()));
    assert_eq!(store.last().await, Some((2, "two".to_string())));

    // Insert a larger element, check that is now last
    assert_eq!(store.get(4).await, None);
    store.insert(4, "four".to_string()).await;
    assert_eq!(store.len().await, 2);
    assert_eq!(store.get(4).await, Some("four".to_string()));
    assert_eq!(store.last().await, Some((4, "four".to_string())));

    // Insert a smaller element check that largest element remains unchanged
    assert_eq!(store.get(3).await, None);
    store.insert(3, "three".to_string()).await;
    assert_eq!(store.len().await, 3);
    assert_eq!(store.get(3).await, Some("three".to_string()));
    assert_eq!(store.last().await, Some((4, "four".to_string())));

    // Remove a smaller element and check that the larger element remains unchanged
    assert!(store.delete(3).await);
    assert_eq!(store.len().await, 2);
    assert_eq!(store.get(3).await, None);
    assert!(!store.delete(3).await);
    assert_eq!(store.last().await, Some((4, "four".to_string())));

    // Remove the largest element and check that a smaller element is now the largest
    assert!(store.delete(4).await);
    assert_eq!(store.len().await, 1);
    assert_eq!(store.get(4).await, None);
    assert!(!store.delete(4).await);
    assert_eq!(store.last().await, Some((2, "two".to_string())));
}

pub async fn async_searchable_store_test<S: AsyncSearchableStore<Key = u64, Value = String>>(
    store: S,
) {
    assert!(store.is_empty().await);

    // Insert some element out of ourder
    store.insert(4, "four".to_string()).await;
    store.insert(3, "three".to_string()).await;
    store.insert(2, "two".to_string()).await;

    // Check that the search scanns through the store correcty
    let mut idx = 2;
    store
        .filter(|key, _| {
            assert_eq!(key, &idx);
            idx += 1;
            true
        })
        .await;

    // Find a specific element
    let find = store.find(|key, _| key == &3).await;
    assert_eq!(find, Some((3, "three".to_string())));

    assert!(store.delete(3).await);
    let find = store.find(|key, _| key == &3).await;
    assert_eq!(find, None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use luct_core::store::{MemoryStore, async_adapter::AsyncAdapter};

    #[tokio::test]
    async fn async_memory_store() {
        let store = AsyncAdapter::new(MemoryStore::<u64, String>::default());
        async_store_test(store).await;
    }

    #[tokio::test]
    async fn async_memory_ordered_store() {
        let store = AsyncAdapter::new(MemoryStore::<u64, String>::default());
        async_ordered_store_test(store).await;
    }

    #[tokio::test]
    async fn async_memory_searchable_store() {
        let store = AsyncAdapter::new(MemoryStore::<u64, String>::default());
        async_searchable_store_test(store).await;
    }
}
