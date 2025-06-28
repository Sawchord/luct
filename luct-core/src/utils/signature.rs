use crate::utils::{
    codec::{CodecError, Decode, Encode},
    vec::CodecVec,
};
use std::{
    io::{Read, Write},
    marker::PhantomData,
};

/// See RFC 5246 4.7
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Signature<T> {
    algorithm: SignatureAndHashAlgorithm,
    signature: CodecVec<u16>,
    inner: PhantomData<T>,
}

impl<T> Encode for Signature<T> {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.algorithm.encode(&mut writer)?;
        self.signature.encode(&mut writer)?;
        Ok(())
    }
}

impl<T> Decode for Signature<T> {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            algorithm: SignatureAndHashAlgorithm::decode(&mut reader)?,
            signature: CodecVec::decode(&mut reader)?,
            inner: PhantomData,
        })
    }
}

// TODO: Implement signature validation

/// See RFC 5246 7.4.1.4.1
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SignatureAndHashAlgorithm {
    pub hash: HashAlgorithm,
    pub signature: SignatureAlgorithm,
}

impl Encode for SignatureAndHashAlgorithm {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.hash.encode(&mut writer)?;
        self.signature.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for SignatureAndHashAlgorithm {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            hash: HashAlgorithm::decode(&mut reader)?,
            signature: SignatureAlgorithm::decode(&mut reader)?,
        })
    }
}

/// See RFC 5246 7.4.1.4.1
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum HashAlgorithm {
    None,
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
}

impl Encode for HashAlgorithm {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let discriminant = match self {
            HashAlgorithm::None => 0,
            HashAlgorithm::Md5 => 1,
            HashAlgorithm::Sha1 => 2,
            HashAlgorithm::Sha224 => 3,
            HashAlgorithm::Sha256 => 4,
            HashAlgorithm::Sha384 => 5,
            HashAlgorithm::Sha512 => 6,
        };
        Ok(writer.write_all(&[discriminant])?)
    }
}

impl Decode for HashAlgorithm {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = vec![0u8];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(HashAlgorithm::None),
            1 => Ok(HashAlgorithm::Md5),
            2 => Ok(HashAlgorithm::Sha1),
            3 => Ok(HashAlgorithm::Sha224),
            4 => Ok(HashAlgorithm::Sha256),
            5 => Ok(HashAlgorithm::Sha384),
            6 => Ok(HashAlgorithm::Sha512),
            x => Err(CodecError::UnknownVariant("HashAlgorithm", x as u64)),
        }
    }
}

/// See RFC 5246 7.4.1.4.1
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SignatureAlgorithm {
    Anonymous,
    Rsa,
    Dsa,
    Ecdsa,
}

impl Encode for SignatureAlgorithm {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let discriminant = match self {
            SignatureAlgorithm::Anonymous => 0,
            SignatureAlgorithm::Rsa => 1,
            SignatureAlgorithm::Dsa => 2,
            SignatureAlgorithm::Ecdsa => 3,
        };
        Ok(writer.write_all(&[discriminant])?)
    }
}

impl Decode for SignatureAlgorithm {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = vec![0u8];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(SignatureAlgorithm::Anonymous),
            1 => Ok(SignatureAlgorithm::Rsa),
            2 => Ok(SignatureAlgorithm::Dsa),
            3 => Ok(SignatureAlgorithm::Ecdsa),
            x => Err(CodecError::UnknownVariant("SignatureAlgorithm", x as u64)),
        }
    }
}
