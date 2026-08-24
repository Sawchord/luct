use luct_core::{
    store::{Hashable, StoreBase, StoreRead},
    tiling::TileId,
    tree::{HashOutput, Node, NodeKey},
};
use std::{
    fmt::{self},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{Client, CtClient};

pub struct TileFetchStore<C> {
    name: String,
    client: CtClient<C>,
    tree_size: AtomicU64,
}

impl<C> fmt::Debug for TileFetchStore<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileFetchStore")
            .field("name", &self.name)
            .finish()
    }
}

impl<C> TileFetchStore<C> {
    pub fn new(name: String, client: CtClient<C>) -> Self {
        TileFetchStore {
            name,
            client,
            tree_size: AtomicU64::new(0),
        }
    }
}

impl<C> TileFetchStore<C> {
    pub fn set_tree_size(&self, tree_size: u64) {
        self.tree_size.store(tree_size, Ordering::Release);
    }
}

impl<C> StoreBase for TileFetchStore<C> {
    type Key = NodeKey;
    type Value = HashOutput;
}

impl<C: Client> StoreRead for TileFetchStore<C> {
    async fn get(&self, key: NodeKey) -> Option<HashOutput> {
        // If not available, calculate which tile should have the value and fetch it
        let tree_size = self.tree_size.load(Ordering::Acquire);
        if tree_size == 0 {
            tracing::error!(
                "Failed to retrieve STH for log {}. Initialize STH before checking inclusions",
                self.name
            );
            return None;
        }

        tracing::trace!("Fetching key {:?} against tree size {}", key, tree_size);
        let nodes = self.fetch_unbalanced_keys(&key, tree_size).await?;

        // Pick the result from the recomputed nodes
        let result = nodes
            .iter()
            .find(|(nk, _)| nk == &key)
            .map(|(_, hash)| *hash)
            .expect("Node was not included in result. This is a bug");

        Some(result)
    }

    async fn len(&self) -> usize {
        self.tree_size.load(Ordering::Acquire) as usize
    }
}

impl<C: Client> TileFetchStore<C> {
    async fn fetch_unbalanced_keys(
        &self,
        key: &NodeKey,
        tree_size: u64,
    ) -> Option<Vec<(NodeKey, [u8; 32])>> {
        let nodes = if key.is_balanced() {
            // If the key is balanced, we know it is contained within exactly one tile.
            // We call `fetch_balanced_tile` to fetch the tile and then recompute the nodes
            tracing::trace!("Fetching balanced key: {:?}", key);
            self.fetch_balanced_keys(key, tree_size).await?
        } else {
            // If the key is unbalanced, we might need to fetch multiple tiles.
            // We split the key into a balanced left part and an unbalanced right part which we fetch recursively
            let (left, right) = key.split();
            tracing::trace!("Fetching balanced key: {:?}", left);
            tracing::trace!("Fetching unbalanced key: {:?}", right);
            let (left_nodes, right_nodes) = futures::join!(
                self.fetch_balanced_keys(&left, tree_size),
                Box::pin(self.fetch_unbalanced_keys(&right, tree_size)),
            );

            let mut left_nodes = left_nodes?;
            let mut right_nodes = right_nodes?;

            let left_hash = left_nodes.iter().find(|(key, _)| key == &left)?.1;
            let right_hash = right_nodes.iter().find(|(key, _)| key == &right)?.1;

            let hash = Node {
                left: left_hash,
                right: right_hash,
            }
            .hash();

            left_nodes.append(&mut right_nodes);
            left_nodes.push((key.clone(), hash));

            tracing::trace!("Fetched unbalanced key: {:?}", key);
            left_nodes
        };

        tracing::trace!("Fetched {} nodes", nodes.len());
        Some(nodes)
    }

    async fn fetch_balanced_keys(
        &self,
        key: &NodeKey,
        tree_size: u64,
    ) -> Option<Vec<(NodeKey, [u8; 32])>> {
        let tile_id = TileId::from_node_key(key, tree_size)?;
        let tile = self.client.get_tile(tile_id.clone()).await;

        if tile.is_err() {
            tracing::error!("Failed to fetch tile {:?}, reason: {:?}", tile_id, tile);
        }

        let tile = tile.ok()?;
        let nodes = tile.recompute_node_keys();

        tracing::trace!("Fetched balanced key: {:?}", key);
        Some(nodes)
    }
}
