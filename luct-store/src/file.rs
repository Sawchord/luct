use crate::{StringStoreKey, StringStoreValue};
use futures::{FutureExt, future::join_all};
use luct_core::store::{
    AsyncOrderedStoreRead, AsyncSearchableStoreRead, AsyncStoreRead, AsyncStoreWrite, StoreBase,
};
use std::{marker::PhantomData, path::PathBuf, sync::Arc};
use tokio::{io::AsyncWriteExt, sync::RwLock};

// TODO: Log errors

/// Implementation of [`Store`](luct_core::store::Store) that is backed by a directory.
///
/// # Description
/// [`FilesystemStore`] used a directory named after the store and stores the keys as files.
/// It requires both [`StringStoreKey`] for keys and [`StringStoreValue`] for values, since
/// it stores the values as [`Strings`](String) as well.
///
/// This implementation is not efficient in any way.
/// It is fast enough for CLI usage, since the amount of data processed there is relatively small.
/// Also, storing data as [`Stings`](String) in files makes debugging and understanding what data has
/// been stored very easy.
///
/// Searching through the store is done by scanning through the directory, which is very slow.
///
/// # Caution
/// There is no locking or checking that each path is instanciated only once.
/// You must be careful not to instanciate two stores at the same location.
///
/// Also starting a program that uses the store twice may lead to problems.
/// This is supposed to be used for simple applications and CLI.
/// You may need a database storage backend for more complex applications such as log servers.
#[derive(Clone, Debug)]
pub struct FilesystemStore<K, V> {
    _kv: PhantomData<(K, V)>,
    path: PathBuf,

    access: Arc<RwLock<()>>,
}

impl<K, V> FilesystemStore<K, V> {
    /// Create a new [`FilesystemStore`], at the `path`
    pub fn new(path: PathBuf) -> FilesystemStore<K, V> {
        std::fs::create_dir_all(&path)
            .inspect_err(|err| {
                tracing::error!(
                    "Failed to create necessary directory {:?} for filesystem store, err: {:?}",
                    path,
                    err,
                )
            })
            .expect("Failed to set up filesystem store");

        Self {
            _kv: PhantomData,
            path,
            access: Arc::new(RwLock::new(())),
        }
    }
}

impl<K: StringStoreKey, V: StringStoreValue> FilesystemStore<K, V> {
    async fn get_sorted_keys(&self) -> Option<Vec<K>> {
        let mut paths = tokio::fs::read_dir(&self.path).await.ok()?;
        let mut keys = Vec::<K>::new();

        while let Some(dir_entry) = paths.next_entry().await.unwrap() {
            match K::deserialize_key(&dir_entry.file_name().into_string().unwrap()) {
                Some(key) => keys.push(key),
                None => tracing::error!("Failed to deserialize a key (get_sorted_keys)",),
            };
        }

        keys.sort();

        Some(keys)
    }
}

impl<K, V> StoreBase for FilesystemStore<K, V> {
    type Key = K;
    type Value = V;
}

impl<K: StringStoreKey, V: StringStoreValue> AsyncStoreRead for FilesystemStore<K, V> {
    async fn get(&self, key: K) -> Option<V> {
        let _lock = self.access.read().await;
        let data = tokio::fs::read_to_string(self.path.join(key.serialize_key()))
            .await
            .ok()?;
        let value = V::deserialize_value(&data)?;
        Some(value)
    }

    async fn len(&self) -> usize {
        let _lock = self.access.read().await;
        match tokio::fs::read_dir(&self.path).await {
            Ok(mut paths) => {
                let mut count = 0;
                while let Some(_path) = paths.next_entry().await.unwrap() {
                    count += 1;
                }
                count
            }
            Err(_) => 0,
        }
    }
}

impl<K, V> AsyncStoreWrite for FilesystemStore<K, V>
where
    K: StringStoreKey,
    V: StringStoreValue,
{
    async fn insert(&self, key: K, value: V) {
        let _lock = self.access.write().await;
        let store_path = self.path.join(key.serialize_key());

        match tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&store_path)
            .await
        {
            Ok(mut file) => {
                file.write_all(value.serialize_value().as_bytes())
                    .await
                    .unwrap();
                tracing::debug!("Wrote key to {:?}", store_path);
            }
            Err(err) => tracing::error!("Failed to write to path {:?}, err {:?}", store_path, err),
        };
    }

    async fn delete(&self, key: K) -> bool {
        let _lock = self.access.write().await;
        tokio::fs::remove_file(self.path.join(key.serialize_key()))
            .await
            .is_ok()
    }
}

impl<K, V> AsyncOrderedStoreRead for FilesystemStore<K, V>
where
    K: StringStoreKey,
    V: StringStoreValue,
{
    async fn last(&self) -> Option<(K, V)> {
        let _lock = self.access.read().await;
        let keys = self.get_sorted_keys().await?;

        // If the last one exists, try to read the value
        let key = keys.last().cloned()?;
        let data = tokio::fs::read_to_string(self.path.join(key.serialize_key()))
            .await
            .ok()?;
        let val = V::deserialize_value(&data)?;

        Some((key, val))
    }
}

impl<K, V> AsyncSearchableStoreRead for FilesystemStore<K, V>
where
    K: StringStoreKey,
    V: StringStoreValue,
{
    async fn filter(&self, mut pred: impl FnMut(&K, &V) -> bool) -> Vec<(K, V)> {
        let _lock = self.access.read().await;
        let Some(keys) = self.get_sorted_keys().await else {
            return vec![];
        };

        let x = keys.into_iter().map(|key| {
            tokio::fs::read_to_string(self.path.join(key.serialize_key())).map(|val| {
                val.ok()
                    .and_then(|val| V::deserialize_value(&val).map(|val| (key, val)))
            })
        });

        join_all(x)
            .await
            .into_iter()
            .flatten()
            .filter(|(key, val)| pred(key, val))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luct_test::store::{
        async_ordered_store_test, async_searchable_store_test, async_store_test,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn async_filesystem_store() {
        let dir = TempDir::new().unwrap();

        let store = FilesystemStore::<u64, String>::new(dir.path().to_owned());
        async_store_test(store).await;
    }

    #[tokio::test]
    async fn async_filesystem_ordered_store() {
        let dir = TempDir::new().unwrap();

        let store = FilesystemStore::<u64, String>::new(dir.path().to_owned());
        async_ordered_store_test(store).await;
    }

    #[tokio::test]
    async fn async_filesystem_searchable_store() {
        let dir = TempDir::new().unwrap();

        let store = FilesystemStore::<u64, String>::new(dir.path().to_owned());
        async_searchable_store_test(store).await;
    }
}
