use crate::{HashOutput, log::ScannerLogInner};
use luct_client::Client;
use luct_core::{
    store::{AsyncStore, Store},
    tiling::TileId,
    tree::NodeKey,
};
use std::sync::Arc;

struct TileFetchStore<C> {
    node_cache: Box<dyn Store<NodeKey, HashOutput>>,
    log: Arc<ScannerLogInner<C>>,
}

impl<C> AsyncStore<NodeKey, HashOutput> for TileFetchStore<C>
where
    C: Client,
{
    async fn insert(&self, _: NodeKey, _: HashOutput) {
        unimplemented!("It is not possible to insert nodes in a tile fetch store")
    }

    async fn get(&self, key: NodeKey) -> Option<HashOutput> {
        // First, try to get the node from the cache
        if let Some(value) = self.node_cache.get(&key) {
            return Some(value);
        }

        // If not available, calculate which tile should have the value and fetch it
        let tree_size = self.log.sth_store.last()?.1.tree_size();
        let tile_id = TileId::from_node_key(&key, tree_size)?;
        let tile = self.log.client.get_tile(tile_id).await.ok()?;
        let nodes = tile.recompute_node_keys();

        // Pick the result from the recomputed nodes
        let result = nodes
            .iter()
            .find(|(nk, _)| nk == &key)
            .map(|(_, hash)| *hash);

        // Put the rest of the nodes into the cache
        nodes
            .into_iter()
            .for_each(|(key, hash)| self.node_cache.insert(key, hash));

        result
    }

    async fn len(&self) -> usize {
        self.log
            .sth_store
            .last()
            .map(|last| last.1.tree_size() as usize)
            .unwrap_or(0)
    }
}
