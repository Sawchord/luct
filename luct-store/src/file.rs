use luct_core::store::{OrderedStoreRead, StoreRead, StoreWrite};
use std::{
    fs::OpenOptions,
    io::Write,
    marker::PhantomData,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{StringStoreKey, StringStoreValue};

#[derive(Clone)]
pub struct FilesystemStore<K, V> {
    _kv: PhantomData<(K, V)>,
    path: PathBuf,
    access: Arc<Mutex<()>>,
}

impl<K, V> FilesystemStore<K, V> {
    pub fn new(path: PathBuf) -> FilesystemStore<K, V> {
        Self {
            _kv: PhantomData,
            path,
            access: Arc::new(Mutex::new(())),
        }
    }
}

impl<K: StringStoreKey, V: StringStoreValue> StoreRead<K, V> for FilesystemStore<K, V> {
    fn get(&self, key: &K) -> Option<V> {
        let _lock = self.access.lock().unwrap();
        match std::fs::read_to_string(self.path.join(key.serialize_key())) {
            Ok(data) => V::deserialize_value(&data),
            Err(_) => None,
        }
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
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(self.path.join(key.serialize_key()))
        {
            file.write_all(value.serialize_value().as_bytes()).unwrap()
        }
    }
}

impl<K: StringStoreKey, V: StringStoreValue> OrderedStoreRead<K, V> for FilesystemStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        let _lock = self.access.lock().unwrap();

        let paths = std::fs::read_dir(&self.path).ok()?;

        // Read the directory to file keys
        let mut keys = paths
            .filter_map(|path| match path {
                Ok(dir_entry) => Some(K::deserialize_key(
                    &dir_entry.file_name().into_string().unwrap(),
                ))
                .flatten(),
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        // Sort
        keys.sort();

        // If the last one exists, try to read the value
        let key = keys.last().cloned()?;
        let data = std::fs::read_to_string(self.path.join(key.serialize_key())).ok()?;
        let val = V::deserialize_value(&data)?;

        Some((key, val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luct_test::store::{ordered_store_test, store_test};
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
}
