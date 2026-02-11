use crate::utils::{
    append_vec::SizedAppendVec,
    codec::{CodecError, Decode, Encode},
    codec_vec::CodecVec,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    ops::Deref,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtExtensions(SizedAppendVec<CtExtension>);

impl CtExtensions {
    pub fn leaf_index(&self) -> Option<LeafIndex> {
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
        Ok(Self(SizedAppendVec::decode(reader)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CtExtension {
    LeafIndex(LeafIndex),
    Unknown(u8, CodecVec<u16>),
}

impl Encode for CtExtension {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        match self {
            CtExtension::LeafIndex(leaf_index) => {
                writer.write_all(&[0, 0, 5])?;
                leaf_index.encode(&mut writer)?;
            }
            CtExtension::Unknown(discriminant, bytes) => {
                writer.write_all(&[*discriminant])?;
                bytes.encode(&mut writer)?;
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
                let size = u16::decode(&mut reader)?;
                if size != 5 {
                    return Err(CodecError::UnexpectedSize {
                        read: size as usize,
                        expected: 5,
                    });
                }
                let leaf_index = LeafIndex::decode(&mut reader)?;
                Ok(Self::LeafIndex(leaf_index))
            }
            discriminant => {
                let bytes = CodecVec::decode(&mut reader)?;
                Ok(Self::Unknown(discriminant, bytes))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LeafIndex(u64);

impl Deref for LeafIndex {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
    use rand::{RngExt, rng};
    use std::io::Cursor;

    /// Test that empty bytes parses correctly
    #[test]
    fn empty_extension_serializes() {
        let mut reader = Cursor::new(vec![0, 0]);
        let extensions = CtExtensions::decode(&mut reader).unwrap();
        assert!(extensions.0.as_ref().is_empty());
    }

    #[test]
    fn empty_extension_deserializes() {
        let mut writer = Cursor::new(vec![]);
        CtExtensions::default().encode(&mut writer).unwrap();
        assert_eq!(writer.into_inner(), vec![0, 0]);
    }

    const NUM_EXTENSIONS: usize = 10;

    #[test]
    fn ct_unknown_extensions_roundtrip() {
        let mut vec: Vec<u8> = vec![0, 0];
        for i in 1..=NUM_EXTENSIONS {
            add_random_extension(&mut vec, i * 10);
        }
        set_size_in_vec(&mut vec);

        let mut reader = Cursor::new(&vec);
        let extensions = CtExtensions::decode(&mut reader).unwrap();
        assert_eq!(extensions.0.as_ref().len(), NUM_EXTENSIONS);

        let mut writer = Cursor::new(vec![]);
        extensions.encode(&mut writer).unwrap();

        assert_eq!(vec, writer.into_inner());
    }

    #[test]
    fn parse_leaf_index_extension() {
        let mut vec: Vec<u8> = vec![0, 0];
        for i in 1..=1 {
            let size = i * 10;
            add_random_extension(&mut vec, size);
        }

        vec.extend_from_slice(&[0, 0, 5, 0, 0, 0, 1, 1]);

        for i in 2..=2 {
            let size = i * 10;
            add_random_extension(&mut vec, size);
        }
        set_size_in_vec(&mut vec);

        let mut reader = Cursor::new(&vec);
        let extensions = CtExtensions::decode(&mut reader).unwrap();
        assert_eq!(extensions.0.as_ref().len(), 3);
        assert_eq!(extensions.leaf_index(), Some(LeafIndex(257)));

        let mut writer = Cursor::new(vec![]);
        extensions.encode(&mut writer).unwrap();

        assert_eq!(vec, writer.into_inner());
    }

    fn add_random_extension(vec: &mut Vec<u8>, size: usize) {
        vec.push(rng().random_range(1u8..=255));
        vec.extend_from_slice(&(size as u16).to_be_bytes());
        vec.extend(rng().random_iter::<u8>().take(size));
    }

    fn set_size_in_vec(vec: &mut [u8]) {
        let len = vec.len() as u16 - 2;
        let len = len.to_be_bytes();
        vec[0] = len[0];
        vec[1] = len[1];
    }
}
