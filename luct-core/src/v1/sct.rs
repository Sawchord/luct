use crate::{
    Version,
    utils::{
        codec::{CodecError, Decode, Encode},
        metered::MeteredRead,
        signature::Signature,
        vec::CodecVec,
    },
    v1::{LogEntry, SignatureType},
};
use std::io::{Cursor, ErrorKind, IoSlice, Read, Write};

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SctList(Vec<SignedCertificateTimestamp>);

impl SctList {
    #[allow(dead_code)]
    pub fn new(scts: Vec<SignedCertificateTimestamp>) -> Self {
        Self(scts)
    }

    pub fn into_inner(self) -> Vec<SignedCertificateTimestamp> {
        self.0
    }
}

impl Encode for SctList {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        let mut bytes = 0;
        let mut encoded_scts = vec![];
        for sct in &self.0 {
            let mut buf = Cursor::new(vec![0, 0]);
            buf.set_position(2);

            sct.encode(&mut buf)?;
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

impl Decode for SctList {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let length = u16::decode(&mut reader)?.into();
        let mut scts = vec![];

        let mut reader = MeteredRead::new(reader);

        while reader.get_meter() < length {
            let _len = u16::decode(&mut reader)?;
            let sct = SignedCertificateTimestamp::decode(&mut reader)?;
            scts.push(sct);
        }

        Ok(Self(scts))
    }
}

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedCertificateTimestamp {
    sct_version: Version,
    id: [u8; 32],
    timestamp: u64,
    extensions: CodecVec<u16>,
    signature: Signature<CertificateTimeStamp>,
}

impl SignedCertificateTimestamp {
    pub fn log_id(&self) -> [u8; 32] {
        self.id
    }
}

impl Encode for SignedCertificateTimestamp {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.sct_version.encode(&mut writer)?;
        self.id.encode(&mut writer)?;
        self.timestamp.encode(&mut writer)?;
        self.extensions.encode(&mut writer)?;
        self.signature.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for SignedCertificateTimestamp {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        Ok(Self {
            sct_version: Version::decode(&mut reader)?,
            id: <[u8; 32]>::decode(&mut reader)?,
            timestamp: u64::decode(&mut reader)?,
            extensions: CodecVec::decode(&mut reader)?,
            signature: Signature::decode(&mut reader)?,
        })
    }
}

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CertificateTimeStamp {
    sct_version: Version,
    // SignatureType signature_type = certificate_timestamp;
    timestamp: u64,
    entry: LogEntry,
    extensions: CodecVec<u16>,
}

impl Encode for CertificateTimeStamp {
    fn encode(&self, mut writer: impl Write) -> Result<(), CodecError> {
        self.sct_version.encode(&mut writer)?;
        SignatureType::CertificateTimeStamp.encode(&mut writer)?;
        self.timestamp.encode(&mut writer)?;
        self.entry.encode(&mut writer)?;
        self.extensions.encode(&mut writer)?;
        Ok(())
    }
}

impl Decode for CertificateTimeStamp {
    fn decode(mut reader: impl Read) -> Result<Self, CodecError> {
        let sct_version = Version::decode(&mut reader)?;
        let signature_type = SignatureType::decode(&mut reader)?;
        match signature_type {
            SignatureType::TreeHash => return Err(CodecError::UnexpectedVariant),
            SignatureType::CertificateTimeStamp => (),
        }
        let timestamp = u64::decode(&mut reader)?;
        let entry = LogEntry::decode(&mut reader)?;
        let extensions = CodecVec::decode(&mut reader)?;

        Ok(Self {
            sct_version,
            timestamp,
            entry,
            extensions,
        })
    }
}
