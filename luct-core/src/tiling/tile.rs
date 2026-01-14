use itertools::Itertools;

use crate::{
    store::Hashable,
    tiling::index_to_url,
    tree::{HashOutput, Node, NodeKey},
};
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

    /// Returns the [`Url`](url::Url) path, at which this tile should be found
    ///
    /// Append this path to the `tile_url`, to get the full path.
    pub fn as_url(&self) -> String {
        let index_url = index_to_url(self.index);

        match self.partial {
            Some(partial) => format!("/tile/{}/{}.p/{}", self.level, index_url, partial),
            None => format!("/tile/{}/{}", self.level, index_url),
        }
    }

    /// Create a [`Tile`], by adding the data to this [`TileId`]
    ///
    /// # Returns:
    ///
    /// - `None`: If the length of the data is not a multiple if 32
    /// - `Some(Tile)` otherwise
    pub fn with_data(self, data: Vec<u8>) -> Option<Tile> {
        if !data.len().is_multiple_of(32) {
            return None;
        }

        Some(Tile { id: self, data })
    }

    /// Returns `true`, if this [`TileId`] is partial, `false` otherwise
    pub fn is_partial(&self) -> bool {
        self.partial.is_some()
    }

    /// Turn a partial [`TileId`] into one that is not partial
    ///
    /// Does nothing if [`TileId`] is already partial.
    pub fn into_unpartial(mut self) -> Self {
        self.partial = None;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    id: TileId,
    data: Vec<u8>,
}

impl Tile {
    /// Return the [`TileId`] of this [`Tile`]
    pub fn id(&self) -> &TileId {
        &self.id
    }

    /// Recomputes the [`NodeKeys`](NodeKey) contained within this tile
    pub fn recompute_node_keys(&self) -> Vec<(NodeKey, HashOutput)> {
        // Get the initial Node keys
        let steps = 2u64.pow(8 * self.id.level as u32);
        let tile_width = 256 * steps;
        let tile_start = self.id.index * tile_width;

        let mut nodes: Vec<(NodeKey, HashOutput)> = vec![];
        let mut result: Vec<(NodeKey, HashOutput)> = vec![];

        // Add the base nodes to the output
        for idx in 0..self.data.len() / 32 {
            nodes.push((
                NodeKey {
                    start: tile_start + (idx as u64) * tile_width,
                    end: tile_start + ((idx as u64) + 1) * tile_width,
                },
                self.data[32 * idx..32 * (idx + 1)].try_into().unwrap(),
            ));
        }

        // TODO: Recompute higher nodes
        let mut nodes_added = nodes.len();

        // If we added only one node to the output, we are done
        while nodes_added > 1 {
            nodes_added = 0;

            let end_node = if nodes.len() % 2 == 1 {
                Some(nodes.pop().unwrap())
            } else {
                None
            };

            let new_nodes: Vec<(NodeKey, HashOutput)> = nodes
                .drain(..)
                .chunks(2)
                .into_iter()
                .map(|mut nodes| {
                    let left = nodes.next().unwrap();
                    let right = nodes.next().unwrap();

                    let new_key = left.0.merge(&right.0).unwrap();
                    let new_hash = Node {
                        left: left.1,
                        right: right.1,
                    }
                    .hash();

                    result.push(left);
                    result.push(right);

                    (new_key, new_hash)
                })
                .collect();
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_url() {
        assert_eq!(&tile_id(0, 1, None).as_url(), "/tile/0/001");
        assert_eq!(
            &tile_id(1, 10987654321, None).as_url(),
            "/tile/1/x010/x987/x654/321"
        );
        assert_eq!(
            &tile_id(3, 1234, Some(128)).as_url(),
            "/tile/3/x001/234.p/128"
        );
    }

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
