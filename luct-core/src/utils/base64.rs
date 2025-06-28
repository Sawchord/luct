use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Base64<T>(pub T);

impl<T> Deref for Base64<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Base64<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Base64<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: AsRef<[u8]>> Serialize for Base64<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = BASE64_STANDARD.encode(self.0.as_ref());
        serializer.serialize_str(&encoded)
    }
}

impl<'de, T: From<Vec<u8>>> Deserialize<'de> for Base64<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = <String>::deserialize(deserializer)?;
        BASE64_STANDARD
            .decode(encoded)
            .map_err(serde::de::Error::custom)
            .map(|v| Base64(T::from(v)))
    }
}
