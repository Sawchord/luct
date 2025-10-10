mod file;
pub use file::FilesystemStore;
use luct_core::{
    Fingerprint,
    v1::{SignedCertificateTimestamp, SignedTreeHead},
};

pub trait StringStoreKey: Clone + Ord + Send + 'static {
    fn serialize_key(&self) -> String;
    fn deserialize_key(key: &str) -> Option<Self>;
}

pub trait StringStoreValue: Clone + Send + 'static {
    fn serialize_value(&self) -> String;
    fn deserialize_value(value: &str) -> Option<Self>;
}

impl StringStoreKey for u64 {
    fn serialize_key(&self) -> String {
        self.to_string()
    }

    fn deserialize_key(key: &str) -> Option<Self> {
        key.parse().ok()
    }
}

impl StringStoreKey for [u8; 32] {
    fn serialize_key(&self) -> String {
        hex::encode(self)
    }

    fn deserialize_key(key: &str) -> Option<Self> {
        hex::decode(key)
            .map(|val| val.try_into().ok())
            .ok()
            .flatten()
    }
}

impl StringStoreKey for Fingerprint {
    fn serialize_key(&self) -> String {
        self.0.serialize_key()
    }

    fn deserialize_key(key: &str) -> Option<Self> {
        <[u8; 32]>::deserialize_key(key).map(Fingerprint)
    }
}

impl StringStoreValue for () {
    fn serialize_value(&self) -> String {
        String::new()
    }

    fn deserialize_value(value: &str) -> Option<Self> {
        match value {
            "" => Some(()),
            _ => None,
        }
    }
}

impl StringStoreValue for SignedTreeHead {
    fn serialize_value(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn deserialize_value(value: &str) -> Option<Self> {
        serde_json::from_str(value).ok()
    }
}

impl StringStoreValue for SignedCertificateTimestamp {
    fn serialize_value(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn deserialize_value(value: &str) -> Option<Self> {
        serde_json::from_str(value).ok()
    }
}

// TODO: Implement RedbStore
