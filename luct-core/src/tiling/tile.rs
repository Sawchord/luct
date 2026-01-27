use crate::{
    store::Hashable,
    tiling::index_to_url,
    tree::{HashOutput, Node, NodeKey},
};
use std::{num::NonZeroU8, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileId {
    level: u8,
    index: u64,
    partial: Option<NonZeroU8>,
    tree_size: u64,
}

impl TileId {
    /// Returns the [`TileId`] of the tile, which contains the [`NodeKey`].
    ///
    /// The `tree_height` is used to calculate, wether the tile in question should be partial or not.
    pub fn from_node_key(key: &NodeKey, tree_size: u64) -> Option<Self> {
        // Compute from the size of the node key, what level of tiles we expect the
        let level = key.size().next_power_of_two().ilog2();
        let level: u8 = (level / 8).try_into().unwrap();

        // Compute the size of the base node keys of the tile, i.e. the nodes that are actually contained in the tiles.
        let steps: u64 = 2u64.pow(8 * level as u32);
        let tile_width = 256 * steps;

        // Compute the index of the tile, that should contain the node
        let index = key.start / tile_width;

        // Check if we need to fetch a partial tile, and if so, compute it's size
        let tile_end = (index + 1) * tile_width;
        let partial = if tile_end < tree_size {
            None
        } else {
            let partial = tree_size % tile_width;
            let partial: u8 = (partial >> (8 * level)).try_into().unwrap();

            Some(NonZeroU8::new(partial).unwrap())
        };

        Some(Self {
            level,
            index,
            partial,
            tree_size,
        })
    }

    /// Returns the [`Url`](url::Url) path, at which this tile should be found
    ///
    /// Append this path to the `tile_url`, to get the full path.
    pub fn as_url(&self) -> String {
        let index_url = index_to_url(self.index);

        match self.partial {
            Some(partial) => format!("tile/{}/{}.p/{}", self.level, index_url, partial),
            None => format!("tile/{}/{}", self.level, index_url),
        }
    }

    /// Create a [`Tile`], by adding the data to this [`TileId`]
    ///
    /// # Returns:
    ///
    /// - `None`: If the length of the data is not a multiple if 32
    /// - `Some(Tile)` otherwise
    pub fn with_data(self, data: Arc<Vec<u8>>) -> Option<Tile> {
        if !data.len().is_multiple_of(32) {
            return None;
        }

        // Check that length actually matches the partial value
        let expected_len = match self.partial {
            Some(val) => usize::from(u8::from(val)),
            None => 256usize,
        };
        if data.len() != expected_len * 32 {
            // TODO: Introduce an error type for size mismatch
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
    data: Arc<Vec<u8>>,
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

        // Add the base nodes to the output
        for idx in 0..self.data.len() / 32 {
            let node = (
                NodeKey {
                    start: tile_start + (idx as u64) * steps,
                    end: tile_start + ((idx as u64) + 1) * steps,
                },
                self.data[32 * idx..32 * (idx + 1)].try_into().unwrap(),
            );

            nodes.push(node);
        }

        let mut start_idx = 0;
        while start_idx < nodes.len() - 1 {
            let end_node = if nodes.len() % 2 == 1 {
                Some(nodes.pop().unwrap())
            } else {
                None
            };

            for idx in (start_idx..nodes.len()).step_by(2) {
                let left = &nodes[idx];
                let right = &nodes[idx + 1];

                nodes.push((
                    left.0.merge(&right.0).unwrap(),
                    Node {
                        left: left.1,
                        right: right.1,
                    }
                    .hash(),
                ));

                start_idx += 2;
            }

            if let Some(node) = end_node {
                nodes.push(node)
            }
        }

        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{Rng, rng};

    #[test]
    fn as_url() {
        assert_eq!(&tile_id(0, 1, None, 0).as_url(), "tile/0/001");
        assert_eq!(
            &tile_id(1, 10987654321, None, 0).as_url(),
            "tile/1/x010/x987/x654/321"
        );
        assert_eq!(
            &tile_id(3, 1234, Some(128), 0).as_url(),
            "tile/3/x001/234.p/128"
        );
    }

    #[test]
    fn into_tile_id() {
        assert_eq!(
            tile_id_from_node_key(4, 5, 70000),
            tile_id(0, 0, None, 70000),
        );
        assert_eq!(
            tile_id_from_node_key(270, 271, 70000),
            tile_id(0, 1, None, 70000),
        );

        assert_eq!(
            tile_id_from_node_key(0, 128, 70000),
            tile_id(0, 0, None, 70000),
        );
        assert_eq!(
            tile_id_from_node_key(0, 256, 70000),
            tile_id(1, 0, None, 70000),
        );

        assert_eq!(
            tile_id_from_node_key(69950, 69951, 70000),
            tile_id(0, 273, Some(112), 70000)
        );

        assert_eq!(
            tile_id_from_node_key(1 << 16, 70000, 70000),
            tile_id(1, 1, Some(17), 70000),
        );

        assert_eq!(
            tile_id_from_node_key(0, 70000, 70000),
            tile_id(2, 0, Some(1), 70000),
        );
    }

    #[test]
    fn recompute_node_keys_small_examples() {
        let tile = tile_id(0, 0, Some(1), 0)
            .with_data(random_tile_data(1))
            .unwrap();
        let node_keys = tile.recompute_node_keys();
        assert_node_keys(&node_keys, &[nk(0, 1)]);

        let tile = tile_id(0, 0, Some(3), 0)
            .with_data(random_tile_data(3))
            .unwrap();
        let node_keys = tile.recompute_node_keys();
        assert_node_keys(
            &node_keys,
            &[nk(0, 1), nk(1, 2), nk(0, 2), nk(2, 3), nk(0, 3)],
        );
    }

    // TODO: recompute_node_keys_sizes

    #[test]
    fn fetch_node_keys() {
        let node_key = NodeKey {
            start: 801112064,
            end: 804490383,
        };

        let id = TileId::from_node_key(&node_key, 804490383).unwrap();
        assert_eq!(id, tile_id(2, 47, Some(244), 804490383));

        let tile = id.with_data(random_tile_data(244)).unwrap();
        let node_keys = tile.recompute_node_keys();

        let deb = node_keys
            .iter()
            .map(|(key, _)| (key.clone(), key.size()))
            .collect::<Vec<_>>();
        dbg!(deb);

        assert!(node_keys.iter().any(|(key, _)| key == &node_key));
    }

    fn tile_id_from_node_key(start: u64, end: u64, size: u64) -> TileId {
        TileId::from_node_key(&NodeKey { start, end }, size).unwrap()
    }

    fn tile_id(level: u8, index: u64, partial: Option<u8>, tree_size: u64) -> TileId {
        TileId {
            level,
            index,
            partial: partial.map(|partial| NonZeroU8::new(partial).unwrap()),
            tree_size,
        }
    }

    fn nk(start: u64, end: u64) -> NodeKey {
        NodeKey { start, end }
    }

    fn random_tile_data(size: u8) -> Arc<Vec<u8>> {
        Arc::new(rng().random_iter().take(size as usize * 32).collect())
    }

    fn assert_node_keys(node_keys: &[(NodeKey, HashOutput)], test_keys: &[NodeKey]) {
        assert_eq!(node_keys.len(), test_keys.len());

        node_keys
            .iter()
            .map(|key| &key.0)
            .zip(test_keys)
            .for_each(|(key, test_key)| assert_eq!(key, test_key));
    }
}
