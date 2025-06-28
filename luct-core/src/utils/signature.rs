use crate::utils::{
    codec::{CodecError, Decode, Encode},
    vec::CodecVec,
};
use digest::DynDigest;
use p256::{
    ecdsa::{Signature as EcdsaSignature, VerifyingKey, signature::Verifier},
    pkcs8::DecodePublicKey,
};
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use std::{
    fmt::Display,
    io::{Cursor, Read, Write},
    marker::PhantomData,
};
use thiserror::Error;

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

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SignatureValidationError {
    #[error("The hash algorithm {0} is not supported by the implementation")]
    UnsupportedHashAlgorithm(HashAlgorithm),

    #[error("The signature algorithm {0} is not supported by the implementation")]
    UnsupportedSignatureAlgorithm(SignatureAlgorithm),

    #[error("The key could not be parsed for the specified signature algorithm")]
    MalformedKey,

    #[error("The signature could not be parsed for the specified signautre algorithm")]
    MalformedSignature,

    #[error("The signature verification failed")]
    InvalidSignature,

    #[error("Error encoding a value: {0}")]
    CodecError(#[from] CodecError),
}

impl<T: Encode> Signature<T> {
    pub fn validate(&self, val: &T, key: &[u8]) -> Result<(), SignatureValidationError> {
        let mut data = Cursor::new(vec![]);
        val.encode(&mut data)?;

        let _digest: Box<dyn DynDigest> = match &self.algorithm.hash {
            HashAlgorithm::Sha224 => Box::new(Sha224::new()),
            HashAlgorithm::Sha256 => Box::new(Sha256::new()),
            HashAlgorithm::Sha384 => Box::new(Sha384::new()),
            HashAlgorithm::Sha512 => Box::new(Sha512::new()),
            alg => {
                return Err(SignatureValidationError::UnsupportedHashAlgorithm(
                    alg.clone(),
                ));
            }
        };

        match &self.algorithm.signature {
            SignatureAlgorithm::Ecdsa => {
                let verifying_key = VerifyingKey::from_public_key_der(key)
                    .map_err(|_| SignatureValidationError::MalformedKey)?;

                let signature = EcdsaSignature::from_der(self.signature.as_ref())
                    .map_err(|_| SignatureValidationError::MalformedSignature)?;

                verifying_key
                    .verify(&data.into_inner(), &signature)
                    .map_err(|_| SignatureValidationError::InvalidSignature)?;

                Ok(())
            }
            alg => Err(SignatureValidationError::UnsupportedSignatureAlgorithm(
                alg.clone(),
            )),
        }
    }
}

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
pub enum HashAlgorithm {
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

impl Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::None => write!(f, "None"),
            HashAlgorithm::Md5 => write!(f, "Md5"),
            HashAlgorithm::Sha1 => write!(f, "Sha1"),
            HashAlgorithm::Sha224 => write!(f, "Sha224"),
            HashAlgorithm::Sha256 => write!(f, "Sha256"),
            HashAlgorithm::Sha384 => write!(f, "Sha384"),
            HashAlgorithm::Sha512 => write!(f, "Sha512"),
        }
    }
}

/// See RFC 5246 7.4.1.4.1
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SignatureAlgorithm {
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

impl Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignatureAlgorithm::Anonymous => write!(f, "Anonymous"),
            SignatureAlgorithm::Rsa => write!(f, "Rsa"),
            SignatureAlgorithm::Dsa => write!(f, "Dsa"),
            SignatureAlgorithm::Ecdsa => write!(f, "Ecdsa"),
        }
    }
}
