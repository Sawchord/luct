// TODO: Implement Filestore

use luct_core::store::{IndexedStore, OrderedStore, Store};
use std::{marker::PhantomData, path::PathBuf};

#[derive(Clone)]
pub struct FilesystemStore<K, V> {
    _kv: PhantomData<(K, V)>,
    path: PathBuf,
}

impl<K, V> FilesystemStore<K, V> {
    fn new(path: PathBuf) -> FilesystemStore<K, V> {
        todo!()
    }
}

impl<K, V> Store<K, V> for FilesystemStore<K, V> {
    fn insert(&self, key: K, value: V) {
        todo!()
    }

    fn get(&self, key: &K) -> Option<V> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }
}

impl<K: Ord, V> OrderedStore<K, V> for FilesystemStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        todo!()
    }
}

impl<V> IndexedStore<V> for FilesystemStore<u64, V> {
    fn insert_indexed(&self, value: V) -> u64 {
        todo!()
    }
}
