use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    io::{Cursor, Read, Write},
    ops::{Deref, DerefMut},
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum CodecError {
    #[error("Error of underlying IO: {0}")]
    IoError(std::io::ErrorKind),

    #[error("There is no variant with the discriminant {1} in {0}")]
    UnknownVariant(&'static str, u64),

    #[error("{name} field contained {received} byte (maximum is {max} bytes)")]
    TooLong {
        name: &'static str,
        received: usize,
        max: usize,
    },
}

impl From<std::io::Error> for CodecError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value.kind())
    }
}

pub(crate) trait Encode {
    fn encode(&self, writer: impl Write) -> Result<(), CodecError>;
}

pub(crate) trait Decode
where
    Self: Sized,
{
    fn decode(reader: impl Read) -> Result<Self, CodecError>;
}

pub(crate) struct Codec<T>(T);

impl<T> Deref for Codec<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Codec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Encode> Serialize for Codec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut data = Cursor::new(vec![]);
        self.encode(&mut data).map_err(serde::ser::Error::custom)?;
        data.flush().map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&data.into_inner())
    }
}

impl<'de, T: Decode> Deserialize<'de> for Codec<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = Cursor::new(<Vec<u8>>::deserialize(deserializer)?);
        T::decode(data)
            .map_err(serde::de::Error::custom)
            .map(|v| Self(v))
    }
}
