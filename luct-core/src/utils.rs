use itertools::Itertools;

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
