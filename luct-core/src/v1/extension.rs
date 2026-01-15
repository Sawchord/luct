use crate::utils::{
    append_vec::AppendVec,
    codec::{CodecError, Decode, Encode},
};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtExtensions(AppendVec<CtExtension>);

impl CtExtensions {
    pub fn get_leaf_index(&self) -> Option<LeafIndex> {
        self.0.as_ref().iter().find_map(|ext| match ext {
            CtExtension::LeafIndex(leaf_index) => Some(leaf_index.clone()),
            _ => None,
        })
    }
}

impl Encode for CtExtensions {
    fn encode(&self, writer: impl Write) -> Result<(), CodecError> {
        self.0.encode(writer)
    }
}

impl Decode for CtExtensions {
    fn decode(reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self(AppendVec::decode(reader)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CtExtension {
    LeafIndex(LeafIndex),
    Unknown(u8, Vec<u8>),
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
                writer.write_all(bytes)?
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
                let mut bytes = vec![];
                reader.read_to_end(&mut bytes)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, rng};
    use std::io::Cursor;

    #[test]
    fn ct_unknown_extensions_round_trip() {
        let mut vec: Vec<u8> = vec![];
        for i in 1..=10 {
            let size = i * 10;
            vec.extend_from_slice(&(size as u16).to_be_bytes());
            vec.extend(rng().random_iter::<u8>().take(size));
        }

        let mut reader = Cursor::new(vec);
        let extensions = CtExtensions::decode(&mut reader).unwrap();
        assert_eq!(extensions.0.as_ref().len(), 10);
    }
}
