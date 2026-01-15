use crate::utils::{
    codec::{CodecError, Decode, Encode},
    codec_vec::CodecVecLen,
};
use std::{
    io::{Read, Write},
    ops::Deref,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct U24(u32);

impl Deref for U24 {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Encode for U24 {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let bytes = self.0.to_be_bytes();
        writer.write_all(&bytes[1..4])?;

        Ok(())
    }
}

impl Decode for U24 {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf[1..4])?;
        let val = u32::from_be_bytes(buf);
        Ok(Self(val))
    }
}

impl TryFrom<usize> for U24 {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < 1usize << 24 {
            Ok(Self(value as u32))
        } else {
            Err(())
        }
    }
}

impl TryInto<usize> for U24 {
    type Error = ();

    fn try_into(self) -> Result<usize, Self::Error> {
        self.0.try_into().map_err(|_| ())
    }
}

impl CodecVecLen for U24 {
    const MAX: usize = 3;
}
