use crate::{Version, utils::vec::CodecVec, v1::LogEntry};

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedCertificateTimestamp {
    sct_version: Version,
    id: [u8; 32],
    extensions: CodecVec<u16>,
    // TODO: Signature of Certificate time stamp
}

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CertificateTimeStamp {
    sct_version: Version,
    // SignatureType signature_type = certificate_timestamp;
    timestamp: u64,
    entry: LogEntry,
    extensions: CodecVec<u16>,
}
