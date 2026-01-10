use crate::tree::NodeKey;
use std::num::NonZeroU8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileId {
    level: u8,
    index: u64,
    partial: Option<NonZeroU8>,
}

impl TileId {
    pub fn from_node_key(key: &NodeKey, tree_height: u64) -> Option<Self> {
        todo!()
    }

    pub fn as_url(&self) -> String {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    id: TileId,
    data: Vec<u8>,
}
