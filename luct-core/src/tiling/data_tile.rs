use std::num::NonZeroU8;

use crate::tiling::index_to_url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTileId {
    index: u64,
    partial: Option<NonZeroU8>,
}

impl DataTileId {
    pub fn as_url(&self) -> String {
        let index_url = index_to_url(self.index);

        match self.partial {
            Some(partial) => format!("/tile/data/{}.p/{}", index_url, partial),
            None => format!("/tile/data/{}", index_url),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTile {
    id: DataTileId,
    data: Vec<u8>,
}
