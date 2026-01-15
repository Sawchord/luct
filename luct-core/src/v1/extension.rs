use crate::utils::{
    codec::{CodecError, Decode, Encode},
    codec_vec::CodecVec,
};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtExtensions(Vec<CtExtension>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CtExtension {
    LeafIndex(LeafIndex),
    Unknown(u8, CodecVec<u8>),
}

impl Encode for CtExtension {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        match self {
            CtExtension::LeafIndex(leaf_index) => {
                writer.write_all(&[0])?;
                leaf_index.encode(writer)?;
            }
            CtExtension::Unknown(discriminant, bytes) => {
                writer.write_all(&[*discriminant])?;
                bytes.encode(writer)?
            }
        }

        Ok(())
    }
}

impl Decode for CtExtension {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut byte: [u8; 1] = [0; 1];
        reader.read_exact(&mut byte)?;

        match byte[0] {
            0 => {
                let leaf_index = LeafIndex::decode(reader)?;
                Ok(Self::LeafIndex(leaf_index))
            }
            discriminant => {
                let bytes = CodecVec::decode(reader)?;
                Ok(Self::Unknown(discriminant, bytes))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeafIndex(u64);

impl Encode for LeafIndex {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let bytes: [u8; 8] = self.0.to_be_bytes();
        writer.write_all(&bytes[3..8])?;
        Ok(())
    }
}

impl Decode for LeafIndex {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut bytes: [u8; 8] = [0; 8];
        reader.read_exact(&mut bytes[3..8])?;
        Ok(Self(u64::from_be_bytes(bytes)))
    }
}
