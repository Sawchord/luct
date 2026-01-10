use std::num::NonZeroU8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTileId {
    index: u64,
    partial: Option<NonZeroU8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTile {
    id: DataTileId,
    data: Vec<u8>,
}
