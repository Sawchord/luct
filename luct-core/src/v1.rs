use crate::utils::codec::{CodecError, Decode, Encode};
use std::io::{Read, Write};

mod sth;

/// See RFC 5246 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SignatureType {
    CertificateTimeStamp,
    TreeHash,
}

impl Encode for SignatureType {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let discriminant = match self {
            SignatureType::CertificateTimeStamp => 0,
            SignatureType::TreeHash => 1,
        };
        Ok(writer.write_all(&[discriminant])?)
    }
}

impl Decode for SignatureType {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let mut buf = vec![0u8];
        reader.read_exact(&mut buf)?;

        match buf[0] {
            0 => Ok(SignatureType::CertificateTimeStamp),
            1 => Ok(SignatureType::TreeHash),
            x => Err(CodecError::UnknownVariant("SignatureType", x as u64)),
        }
    }
}
