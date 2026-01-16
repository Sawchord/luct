use crate::utils::codec::{CodecError, Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    io::{Cursor, ErrorKind, IoSlice, Read, Write},
    ops::{Deref, DerefMut},
};

/// A vector that works by appending multiple length delimited structures
///
/// Note that this will continue reading until EOF, so it needs to be at the end of
/// a reader of the reader needs to be limited by [`Read::take`].
///
/// See also [`SizedAppendVec`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AppendVec<I>(Vec<I>);

impl<I> AsRef<[I]> for AppendVec<I> {
    fn as_ref(&self) -> &[I] {
        self.0.as_ref()
    }
}

impl<I> From<Vec<I>> for AppendVec<I> {
    fn from(value: Vec<I>) -> Self {
        Self(value)
    }
}

impl<I> From<AppendVec<I>> for Vec<I> {
    fn from(value: AppendVec<I>) -> Self {
        value.0
    }
}

impl<I> Default for AppendVec<I> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<I: Encode> Encode for AppendVec<I> {
    fn encode(&self, writer: impl Write) -> Result<(), CodecError> {
        let (_, encoded_scts) = encode_to_io_slice(self)?;
        write_all_vec(writer, &encoded_scts)
    }
}

impl<I: Decode> Decode for AppendVec<I> {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut items = vec![];

        loop {
            match I::decode(&mut reader) {
                Ok(item) => items.push(item),
                Err(CodecError::IoError(ErrorKind::UnexpectedEof)) => break,
                Err(err) => return Err(err),
            }
        }

        Ok(Self(items))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SizedVal<I>(I);

impl<I> Deref for SizedVal<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I> DerefMut for SizedVal<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I> From<I> for SizedVal<I> {
    fn from(value: I) -> Self {
        Self(value)
    }
}

impl<I: Encode> Encode for SizedVal<I> {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let mut bytes = Cursor::new(vec![0, 0]);
        bytes.set_position(2);
        self.0.encode(&mut bytes)?;
        let mut bytes = bytes.into_inner();

        let len = bytes.len() - 2;
        let len = len.to_be_bytes();
        bytes[0] = len[0];
        bytes[1] = len[1];

        Ok(writer.write_all(&bytes)?)
    }
}

impl<I: Decode> Decode for SizedVal<I> {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let len = u16::decode(&mut reader)?;

        let mut reader = (&mut reader).take(len.into());
        let item = I::decode(&mut reader)?;

        Ok(Self(item))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SizedAppendVec<I>(AppendVec<I>);

impl<I> AsRef<[I]> for SizedAppendVec<I> {
    fn as_ref(&self) -> &[I] {
        self.0.as_ref()
    }
}

impl<I> From<Vec<I>> for SizedAppendVec<I> {
    fn from(value: Vec<I>) -> Self {
        Self(value.into())
    }
}

impl<I> From<SizedAppendVec<I>> for Vec<I> {
    fn from(value: SizedAppendVec<I>) -> Self {
        value.0.into()
    }
}

impl<I> Default for SizedAppendVec<I> {
    fn default() -> Self {
        Self(AppendVec::default())
    }
}

impl<I: Encode> Encode for SizedAppendVec<I> {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let (bytes, encoded_scts) = encode_to_io_slice(&self.0)?;

        let bytes: u16 = bytes.try_into().map_err(|_| CodecError::UnexpectedSize {
            read: bytes,
            expected: u16::MAX as usize,
        })?;
        bytes.encode(&mut writer)?;

        write_all_vec(writer, &encoded_scts)
    }
}

impl<I: Decode> Decode for SizedAppendVec<I> {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let len = match u16::decode(&mut reader) {
            Ok(len) => len,
            Err(CodecError::IoError(ErrorKind::UnexpectedEof)) => return Ok(Self::default()),
            Err(err) => return Err(err),
        };

        let reader = reader.take(len.into());
        let vec = AppendVec::decode(reader)?;

        Ok(Self(vec))
    }
}

fn encode_to_io_slice<I: Encode>(
    items: &AppendVec<I>,
) -> Result<(usize, VecDeque<Vec<u8>>), CodecError> {
    let mut bytes = 0;
    let mut slices = VecDeque::new();

    for item in &items.0 {
        let mut buf = Cursor::new(vec![0, 0]);
        buf.set_position(2);

        item.encode(&mut buf)?;
        let mut buf = buf.into_inner();

        // Encode the length of the field
        let len = ((buf.len() - 2) as u16).to_be_bytes();
        buf[0] = len[0];
        buf[1] = len[1];

        // Add to byte counter for field size
        bytes += buf.len();
        slices.push_back(buf);
    }

    Ok((bytes, slices))
}

fn write_all_vec(mut writer: impl Write, items: &VecDeque<Vec<u8>>) -> Result<(), CodecError> {
    let mut slices = items
        .iter()
        .map(|buf| IoSlice::new(buf))
        .collect::<Vec<_>>();

    let mut slices: &mut [IoSlice] = &mut slices;
    while !slices.is_empty() {
        match writer.write_vectored(slices) {
            Ok(0) => {
                return Err(CodecError::IoError(std::io::ErrorKind::WriteZero));
            }
            Ok(n) => IoSlice::advance_slices(&mut slices, n),
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}
