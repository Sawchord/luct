use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de, ser};
use std::{
    fmt,
    io::{Cursor, Write},
    ops::{Deref, DerefMut},
};

use crate::utils::codec::{Codec, Decode, Encode};

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

impl Serialize for Base64<Vec<u8>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = BASE64_STANDARD.encode(&self.0);
        serializer.serialize_str(&encoded)
    }
}

struct Base64VecVisitor;

impl<'de> de::Visitor<'de> for Base64VecVisitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "A string containing a base64 encoded value")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_borrowed_str(v)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        BASE64_STANDARD.decode(v).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Base64<Vec<u8>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Base64(deserializer.deserialize_str(Base64VecVisitor)?))
    }
}

impl<T: Encode> Serialize for Base64<Codec<T>> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut data = Cursor::new(vec![]);
        self.encode(&mut data).map_err(ser::Error::custom)?;
        data.flush().map_err(ser::Error::custom)?;

        let encoded = BASE64_STANDARD.encode(data.into_inner());
        serializer.serialize_str(&encoded)
    }
}

impl<'de, T: Decode> Deserialize<'de> for Base64<Codec<T>> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = <String>::deserialize(deserializer)?;
        let decoded = BASE64_STANDARD.decode(encoded).map_err(de::Error::custom)?;
        //dbg!(&decoded[0..100]);

        let data = Cursor::new(decoded);
        T::decode(data)
            .map_err(de::Error::custom)
            .map(|v| Self(Codec(v)))
    }
}
