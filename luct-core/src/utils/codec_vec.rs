use crate::utils::codec::{CodecError, Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
    io::{Read, Write},
    marker::PhantomData,
};

pub(crate) trait CodecVecLen: TryFrom<usize> + TryInto<usize> + Encode + Decode {
    const MAX: usize;
}

impl CodecVecLen for u8 {
    const MAX: usize = 1;
}
impl CodecVecLen for u16 {
    const MAX: usize = 2;
}

impl CodecVecLen for u32 {
    const MAX: usize = 4;
}

impl CodecVecLen for u64 {
    const MAX: usize = 8;
}

/// A vector that gets encoded with a length value
///
/// The length value can be of different type and size, as long as they implement [`CodecVecLen`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde_as]
pub(crate) struct CodecVec<L>(#[serde_as(as = "Bytes")] Vec<u8>, PhantomData<L>);

// TODO: Remove?
impl<L> CodecVec<L> {
    pub(crate) fn len_bytes(&self) -> usize {
        self.0.len()
    }
}

impl<L> AsRef<[u8]> for CodecVec<L> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<L> From<Vec<u8>> for CodecVec<L> {
    fn from(value: Vec<u8>) -> Self {
        Self(value, PhantomData)
    }
}

impl<L: CodecVecLen> Encode for CodecVec<L> {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let len = self.0.len();
        let len: L = self
            .0
            .len()
            .try_into()
            .map_err(|_| CodecError::UnexpectedSize {
                read: len,
                expected: L::MAX,
            })?;
        len.encode(&mut writer)?;

        writer.write_all(&self.0)?;

        Ok(())
    }
}

impl<L: CodecVecLen> Decode for CodecVec<L> {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let len = L::decode(&mut reader)?;
        let len: usize = len.try_into().map_err(|_| CodecError::UnexpectedSize {
            read: 0,
            expected: L::MAX,
        })?;

        let mut buf = vec![0u8; len];
        reader.read_exact(&mut buf)?;
        Ok(Self(buf, PhantomData))
    }
}
