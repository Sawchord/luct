use crate::{tiling::index_to_url, tree::NodeKey};
use std::num::NonZeroU8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileId {
    level: u8,
    index: u64,
    partial: Option<NonZeroU8>,
}

impl TileId {
    /// Returns the [`TileId`] of the tile, which contains the [`NodeKey`].
    ///
    /// The `tree_height` is used to calculate, wether the tile in question should be partial or not.
    pub fn from_node_key(key: &NodeKey, tree_height: u64) -> Option<Self> {
        // Compute from the size of the node key, what level of tiles we expect the
        let level = key.size().next_power_of_two().ilog2() / 8;
        let level: u8 = (level).try_into().unwrap();

        // Compute the size of the base node keys of the tile, i.e. the nodes that are actually contained in the tiles.
        let steps: u64 = 2u64.pow(8 * level as u32);
        let tile_width = 256 * steps;

        // Compute the index of the tile, that should contain the node
        let index = key.start / tile_width;

        // Check if we need to fetch a partial tile, and if so, compute it's size
        let tile_end = (index + 1) * tile_width;
        let partial = if tile_end < tree_height {
            None
        } else {
            let partial = tree_height % tile_width;
            let partial: u8 = (partial >> (8 * level)).try_into().unwrap();

            Some(NonZeroU8::new(partial).unwrap())
        };

        Some(Self {
            level,
            index,
            partial,
        })
    }

    pub fn as_url(&self) -> String {
        let index_url = index_to_url(self.index);

        match self.partial {
            Some(partial) => format!("/tile/{}/{}.p/{}", self.level, index_url, partial),
            None => format!("/tile/{}/{}", self.level, index_url),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    id: TileId,
    data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Test to URL function

    #[test]
    fn into_tile_id() {
        assert_eq!(tile_id_from_node_key(4, 5, 70000), tile_id(0, 0, None));
        assert_eq!(tile_id_from_node_key(270, 271, 70000), tile_id(0, 1, None));

        assert_eq!(tile_id_from_node_key(0, 128, 70000), tile_id(0, 0, None));
        assert_eq!(tile_id_from_node_key(0, 256, 70000), tile_id(1, 0, None));

        assert_eq!(
            tile_id_from_node_key(69950, 69951, 70000),
            tile_id(0, 273, Some(112))
        );

        assert_eq!(
            tile_id_from_node_key(1 << 16, 70000, 70000),
            tile_id(1, 1, Some(17)),
        );

        assert_eq!(
            tile_id_from_node_key(0, 70000, 70000),
            tile_id(2, 0, Some(1)),
        );
    }

    fn tile_id_from_node_key(start: u64, end: u64, size: u64) -> TileId {
        TileId::from_node_key(&NodeKey { start, end }, size).unwrap()
    }

    fn tile_id(level: u8, index: u64, partial: Option<u8>) -> TileId {
        TileId {
            level,
            index,
            partial: partial.map(|partial| NonZeroU8::new(partial).unwrap()),
        }
    }
}
