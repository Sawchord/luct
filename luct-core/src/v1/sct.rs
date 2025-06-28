use crate::{
    Version,
    utils::{u24::U24, vec::CodecVec},
};

/// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SignedCertificateTimestamp {
    sct_version: Version,
    id: [u8; 32],
    extensions: CodecVec<u16>,
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

// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LogEntry {
    X509(CodecVec<U24>),
    PreCert(PreCert),
}

// See RFC 6962 3.2
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PreCert {
    issuer_key_hash: [u8; 32],
    tbs_certificate: CodecVec<U24>,
}
