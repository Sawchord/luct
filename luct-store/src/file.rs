use luct_core::store::{OrderedStoreRead, SearchableStoreRead, StoreRead, StoreWrite};
use std::{
    fs::OpenOptions,
    io::Write,
    marker::PhantomData,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{StringStoreKey, StringStoreValue};

// TODO: Log errors

#[derive(Clone, Debug)]
pub struct FilesystemStore<K, V> {
    _kv: PhantomData<(K, V)>,
    path: PathBuf,
    access: Arc<Mutex<()>>,
}

impl<K, V> FilesystemStore<K, V> {
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
            access: Arc::new(Mutex::new(())),
        }
    }
}

impl<K: StringStoreKey, V: StringStoreValue> FilesystemStore<K, V> {
    fn get_sorted_keys(&self) -> Option<Vec<K>> {
        let paths = std::fs::read_dir(&self.path).ok()?;
        let mut keys = paths
            .filter_map(|path| match path {
                Ok(dir_entry) => Some(K::deserialize_key(
                    &dir_entry.file_name().into_string().unwrap(),
                ))
                .flatten(),
                Err(err) => {
                    tracing::error!(
                        "Failed to deserialize a key (get_sorted_keys) err: {:?}",
                        err
                    );
                    None
                }
            })
            .collect::<Vec<_>>();
        keys.sort();

        Some(keys)
    }
}

impl<K: StringStoreKey, V: StringStoreValue> StoreRead<K, V> for FilesystemStore<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        let _lock = self.access.lock().unwrap();
        let data = std::fs::read_to_string(self.path.join(key.serialize_key())).ok()?;
        let value = V::deserialize_value(&data)?;
        Some(value)
    }

    fn len(&self) -> usize {
        let _lock = self.access.lock().unwrap();
        match std::fs::read_dir(&self.path) {
            Ok(paths) => paths.count(),
            Err(_) => 0,
        }
    }
}

impl<K: StringStoreKey, V: StringStoreValue> StoreWrite<K, V> for FilesystemStore<K, V> {
    fn insert(&self, key: K, value: V) {
        let _lock = self.access.lock().unwrap();
        let store_path = self.path.join(key.serialize_key());

        match OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&store_path)
        {
            Ok(mut file) => {
                file.write_all(value.serialize_value().as_bytes()).unwrap();
                tracing::debug!("Wrote key to {:?}", store_path);
            }
            Err(err) => tracing::error!("Failed to write to path {:?}, err {:?}", store_path, err),
        };
    }

    fn delete(&self, key: &K) -> bool {
        let _lock = self.access.lock().unwrap();
        std::fs::remove_file(self.path.join(key.serialize_key())).is_ok()
    }
}

impl<K: StringStoreKey, V: StringStoreValue> OrderedStoreRead<K, V> for FilesystemStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        let _lock = self.access.lock().unwrap();
        let keys = self.get_sorted_keys()?;

        // If the last one exists, try to read the value
        let key = keys.last().cloned()?;
        let data = std::fs::read_to_string(self.path.join(key.serialize_key())).ok()?;
        let val = V::deserialize_value(&data)?;

        Some((key, val))
    }
}

impl<K: StringStoreKey, V: StringStoreValue> SearchableStoreRead<K, V> for FilesystemStore<K, V> {
    fn filter(&self, mut pred: impl FnMut(&K, &V) -> bool) -> Vec<(K, V)> {
        let _lock = self.access.lock().unwrap();
        let Some(keys) = self.get_sorted_keys() else {
            return vec![];
        };

        keys.into_iter()
            .filter_map(|key| {
                std::fs::read_to_string(self.path.join(key.serialize_key()))
                    .ok()
                    .map(|data| (key, data))
            })
            .filter_map(|(key, data)| V::deserialize_value(&data).map(|val| (key, val)))
            .filter(|(key, val)| pred(key, val))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luct_test::store::{ordered_store_test, searchable_store_test, store_test};
    use tempdir::TempDir;

    #[test]
    fn filesystem_store() {
        let dir = TempDir::new("filesystem_store").unwrap();

        let store = FilesystemStore::<u64, String>::new(dir.path().to_owned());
        store_test(store);
    }

    #[test]
    fn filesystem_ordered_store() {
        let dir = TempDir::new("filesystem_store").unwrap();

        let store = FilesystemStore::<u64, String>::new(dir.path().to_owned());
        ordered_store_test(store);
    }

    #[test]
    fn filesystem_searchable_store() {
        let dir = TempDir::new("filesystem_store").unwrap();

        let store = FilesystemStore::<u64, String>::new(dir.path().to_owned());
        searchable_store_test(store);
    }
}
