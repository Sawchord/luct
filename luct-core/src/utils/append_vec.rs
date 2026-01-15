use crate::utils::{
    codec::{CodecError, Decode, Encode},
    metered::MeteredRead,
};
use std::io::{Cursor, ErrorKind, IoSlice, Read, Write};

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

        let bytes: u16 = bytes.try_into().map_err(|_| CodecError::VectorTooLong {
            received: bytes,
            max: u16::MAX as usize,
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
        let length = u16::decode(&mut reader)?.into();
        let mut scts = vec![];

        let mut reader = MeteredRead::new(reader);

        while reader.get_meter() < length {
            // TODO: Check parsed length and encoded length
            let _len = u16::decode(&mut reader)?;
            let sct = I::decode(&mut reader)?;
            scts.push(sct);
        }

        Ok(Self(scts))
    }
}
