use crate::utils::codec::{CodecError, Decode, Encode};
use std::{
    collections::VecDeque,
    io::{Cursor, ErrorKind, IoSlice, Read, Write},
};

// TODO: Split the functionality into a length delimited version and an unlimited version

/// A vector that works by appending multiple length delimited structures
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl<I: Encode> Encode for AppendVec<I> {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let mut bytes = 0;
        let mut encoded_scts = vec![];
        for item in &self.0 {
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
            encoded_scts.push(buf);
        }
        let mut slices = encoded_scts
            .iter()
            .map(|buf| IoSlice::new(buf))
            .collect::<Vec<_>>();

        let bytes: u16 = bytes.try_into().map_err(|_| CodecError::UnexpectedSize {
            read: bytes,
            expected: u16::MAX as usize,
        })?;

        bytes.encode(&mut writer)?;

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
}

impl<I: Decode> Decode for AppendVec<I> {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut items = vec![];

        loop {
            let len = match u16::decode(&mut reader) {
                Ok(len) => len,
                Err(CodecError::IoError(ErrorKind::UnexpectedEof)) => break,
                Err(err) => return Err(err),
            };

            let mut reader = (&mut reader).take(len.into());
            let sct = I::decode(&mut reader)?;
            items.push(sct);
        }

        Ok(Self(items))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LimitedAppendVec<I>(AppendVec<I>);

impl<I> AsRef<[I]> for LimitedAppendVec<I> {
    fn as_ref(&self) -> &[I] {
        self.0.as_ref()
    }
}

impl<I> From<Vec<I>> for LimitedAppendVec<I> {
    fn from(value: Vec<I>) -> Self {
        Self(value.into())
    }
}

impl<I> From<LimitedAppendVec<I>> for Vec<I> {
    fn from(value: LimitedAppendVec<I>) -> Self {
        value.0.into()
    }
}

impl<I: Encode> Encode for LimitedAppendVec<I> {
    fn encode(&self, writer: impl Write) -> Result<(), CodecError> {
        self.0.encode(writer)
    }
}

impl<I: Decode> Decode for LimitedAppendVec<I> {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let length: u16 = u16::decode(&mut reader)?;
        let reader = reader.take(length.into());

        Ok(Self(AppendVec::decode(reader)?))
    }
}
