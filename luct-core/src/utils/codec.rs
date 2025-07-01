use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    io::{Cursor, Read, Write},
    ops::{Deref, DerefMut},
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CodecError {
    #[error("Error of underlying IO: {0}")]
    IoError(std::io::ErrorKind),

    #[error("Error while decoding DER: {0}")]
    DerError(#[from] x509_cert::der::Error),

    #[error("There is no variant with the discriminant {1} in {0}")]
    UnknownVariant(&'static str, u64),

    #[error("The variant used here is invalid")]
    UnexpectedVariant,

    #[error("A field contained {received} bytes (maximum is {max} bytes)")]
    VectorTooLong { received: usize, max: usize },
    // #[error("A fiedl contained {received} bytes (expected {expected} bytes)")]
    // VectorTooShort { received: usize, expected: usize },
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Codec<T>(pub T);

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

impl<T> From<T> for Codec<T> {
    fn from(value: T) -> Self {
        Self(value)
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

impl Encode for u8 {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        Ok(writer.write_all(&[*self])?)
    }
}

impl Decode for u8 {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

impl Encode for u16 {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let val = self.to_be_bytes();
        writer.write_all(&val)?;
        Ok(())
    }
}

impl Decode for u16 {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
}

impl Encode for u32 {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let val = self.to_be_bytes();
        writer.write_all(&val)?;
        Ok(())
    }
}

impl Decode for u32 {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
}

impl Encode for u64 {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let val = self.to_be_bytes();
        writer.write_all(&val)?;
        Ok(())
    }
}

impl Decode for u64 {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }
}

impl<const N: usize> Encode for [u8; N] {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        Ok(writer.write_all(self)?)
    }
}

impl<const N: usize> Decode for [u8; N] {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8; N];
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}
