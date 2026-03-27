use const_oid::ObjectIdentifier;
use itertools::Itertools;
use x509_cert::{
    der::asn1::{PrintableString, Utf8StringRef},
    name::RdnSequence,
};

pub(crate) mod append_vec;
pub(crate) mod base64;
pub(crate) mod codec;
pub(crate) mod codec_vec;
pub(crate) mod u24;

pub(crate) fn hex_with_colons(data: &[u8]) -> String {
    hex::encode_upper(data)
        .chars()
        .chunks(2)
        .into_iter()
        .map(|mut chunk| format!("{}{}", chunk.next().unwrap(), chunk.next().unwrap()))
        .join(":")
}

pub(crate) fn extract_oid_from_rdn(
    sequence: &RdnSequence,
    oid: ObjectIdentifier,
) -> Option<String> {
    let attr = sequence
        .0
        .iter()
        .flat_map(|inner| inner.0.iter())
        .find(|val| val.oid == oid)?;

    if let Ok(string) = attr.value.decode_as::<PrintableString>() {
        return Some(string.as_str().to_string());
    }

    if let Ok(string) = attr.value.decode_as::<Utf8StringRef>() {
        return Some(string.as_str().to_string());
    }

    None
}
