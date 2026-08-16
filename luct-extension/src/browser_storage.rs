use crate::extension_sys::{Storage, StorageArea, browser};
use std::{fmt, marker::PhantomData};

pub struct BrowserStorage<K, V> {
    _kv: PhantomData<(K, V)>,
    prefix: String,
    storage: StorageArea,
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for BrowserStorage<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrowserStorage")
            .field("_kv", &self._kv)
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl<K, V> BrowserStorage<K, V> {
    pub fn new_local_store(prefix: String) -> Result<Self, String> {
        let storage = browser().with(|browser| browser.storage().local());
        Ok(Self {
            _kv: PhantomData,
            prefix,
            storage,
        })
    }
}

#[cfg(test)]
mod test {}
