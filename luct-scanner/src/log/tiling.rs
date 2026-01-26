use crate::{HashOutput, log::ScannerLogInner};
use luct_client::{Client, ClientError};
use luct_core::{
    CertificateChain, CertificateError,
    store::{AsyncStore, MemoryStore, Store},
    tiling::TileId,
    tree::{NodeKey, Tree, TreeHead},
    v1::{SignedCertificateTimestamp, SignedTreeHead},
};
use std::sync::Arc;

pub(crate) struct TileFetcher<C>(
    Tree<
        TileFetchStore<C>,
        MemoryStore<u64, SignedCertificateTimestamp>,
        SignedCertificateTimestamp,
    >,
);

impl<C> TileFetcher<C> {
    pub(crate) fn new(log: &Arc<ScannerLogInner<C>>) -> Self {
        Self(Tree::new(
            TileFetchStore::new(
                log.clone(),
                Box::new(
                    // TODO: Use an LRU cache
                    MemoryStore::default(),
                ) as _,
            ),
            MemoryStore::default(),
        ))
    }
}

impl<C: Client> TileFetcher<C> {
    pub(crate) async fn check_embdedded_sct_inclusion(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &SignedTreeHead,
        chain: &CertificateChain,
    ) -> Result<(), ClientError> {
        let Some(leaf_index) = sct.leaf_index() else {
            // TODO: Better error type
            return Err(ClientError::AuditProofError);
        };

        println!("Leaf index: {:?}", leaf_index);

        let tree_head = TreeHead::try_from(sth).map_err(|_| ClientError::AuditProofError)?;
        let audit_proof = self
            .0
            .get_audit_proof_async(&tree_head, *leaf_index)
            .await
            // TODO: Better error
            .ok_or(ClientError::AuditProofError)?;

        let leaf = chain
            .as_leaf_v1(sct, true)
            .map_err(|err| ClientError::CertificateError(CertificateError::CodecError(err)))?;
        if !audit_proof.validate(&tree_head, &leaf) {
            return Err(ClientError::AuditProofError);
        }

        Ok(())
    }
}

pub(crate) struct TileFetchStore<C> {
    node_cache: Box<dyn Store<NodeKey, HashOutput>>,
    log: Arc<ScannerLogInner<C>>,
}

impl<C> TileFetchStore<C> {
    pub(crate) fn new(
        log: Arc<ScannerLogInner<C>>,
        node_cache: Box<dyn Store<NodeKey, HashOutput>>,
    ) -> Self {
        Self { node_cache, log }
    }
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

        println!("Fetching tile: {:?}", tile_id);
        let tile = self.log.client.get_tile(tile_id.clone()).await;

        if tile.is_err() {
            println!("Error: {:?}", tile)
        }

        let tile = tile.ok()?;
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

        if result.is_none() {
            println!("Bug: key {:?} was not contained in tile {:?}", key, tile_id);
        }

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
