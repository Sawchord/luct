use crate::{HashOutput, log::ScannerLogInner};
use luct_client::{Client, ClientError};
use luct_core::{
    CertificateChain, CertificateError,
    store::{AsyncStore, Hashable, MemoryStore, Store},
    tiling::{TileId, TilingError},
    tree::{Node, NodeKey, Tree, TreeHead},
    v1::{SignedCertificateTimestamp, SignedTreeHead},
};
use std::{fmt, sync::Arc};

#[derive(Debug)]
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
    #[tracing::instrument(level = "trace")]
    pub(crate) async fn check_embdedded_sct_inclusion(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &SignedTreeHead,
        chain: &CertificateChain,
    ) -> Result<(), ClientError> {
        let Some(leaf_index) = sct.leaf_index() else {
            return Err(TilingError::LeafIndexMissing.into());
        };

        let tree_head = TreeHead::from(sth);

        tracing::debug!(
            "Fetching audit proof for leaf index {:?} for tree size {}",
            leaf_index,
            tree_head.tree_size()
        );

        let audit_proof = self
            .0
            .get_audit_proof_async(&tree_head, *leaf_index)
            .await
            .map_err(TilingError::AuditProofGenerationError)?;

        let leaf = chain
            .as_leaf_v1(sct, true)
            .map_err(|err| ClientError::CertificateError(CertificateError::CodecError(err)))?;

        // TODO: Better error
        if !audit_proof.validate(&tree_head, &leaf) {
            return Err(ClientError::AuditProofError);
        }

        Ok(())
    }

    pub(crate) async fn check_sth_consistency(
        &self,
        old_sth: &SignedTreeHead,
        new_sth: &SignedTreeHead,
    ) -> Result<(), ClientError> {
        let old_tree_head = TreeHead::from(old_sth);
        let new_tree_head = TreeHead::from(new_sth);

        tracing::debug!(
            "Fetching extension proof from tree size {} to {}",
            old_tree_head.tree_size(),
            new_tree_head.tree_size()
        );

        let consistency_proof = self
            .0
            .get_consistency_proof_async(&old_tree_head, &new_tree_head)
            .await
            .map_err(TilingError::ConsistencyProofGenerationError)?;

        // TODO: Better error
        if !consistency_proof.validate(&old_tree_head, &new_tree_head) {
            return Err(ClientError::ConsistencyProofError);
        }

        Ok(())
    }
}

pub(crate) struct TileFetchStore<C> {
    node_cache: Box<dyn Store<NodeKey, HashOutput>>,
    log: Arc<ScannerLogInner<C>>,
}

impl<C> fmt::Debug for TileFetchStore<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileFetchStore").finish()
    }
}

impl<C> TileFetchStore<C> {
    pub(crate) fn new(
        log: Arc<ScannerLogInner<C>>,
        node_cache: Box<dyn Store<NodeKey, HashOutput>>,
    ) -> Self {
        Self { node_cache, log }
    }
}

impl<C: Client> AsyncStore<NodeKey, HashOutput> for TileFetchStore<C> {
    async fn insert(&self, _: NodeKey, _: HashOutput) {
        unimplemented!("It is not possible to insert nodes in a tile fetch store")
    }

    #[tracing::instrument(level = "trace")]
    async fn get(&self, key: NodeKey) -> Option<HashOutput> {
        // First, try to get the node from the cache
        if let Some(value) = self.node_cache.get(&key) {
            return Some(value);
        }

        // If not available, calculate which tile should have the value and fetch it
        let tree_size = match self.log.sth_store.last() {
            Some(sth) => sth.1.tree_size(),
            None => {
                tracing::error!(
                    "Failed to retrieve STH for log {}. Initialize log before checking inclusions",
                    self.log.name
                );
                return None;
            }
        };

        tracing::debug!("Fetching key {:?} against tree size {}", key, tree_size);
        let nodes = self.fetch_unbalanced_keys(&key, tree_size).await?;

        // Pick the result from the recomputed nodes
        let result = nodes
            .iter()
            .find(|(nk, _)| nk == &key)
            .map(|(_, hash)| *hash)
            .expect("Node was not included in result. This is a bug");

        // Put the nodes into the cache
        nodes
            .into_iter()
            .for_each(|(key, hash)| self.node_cache.insert(key, hash));

        Some(result)
    }

    async fn len(&self) -> usize {
        self.log
            .sth_store
            .last()
            .map(|last| last.1.tree_size() as usize)
            .unwrap_or(0)
    }
}

impl<C: Client> TileFetchStore<C> {
    #[tracing::instrument(level = "trace")]
    async fn fetch_unbalanced_keys(
        &self,
        key: &NodeKey,
        tree_size: u64,
    ) -> Option<Vec<(NodeKey, [u8; 32])>> {
        if let Some(value) = self.node_cache.get(key) {
            return Some(vec![(key.clone(), value)]);
        }

        let nodes = if key.is_balanced() {
            // If the key is balanced, we know it is contained within exactly one tile.
            // We call `fetch_balanced_tile` to fetch the tile and then recompute the nodes
            tracing::debug!("Fetching balanced key: {:?}", key);
            self.fetch_balanced_keys(key, tree_size).await?
        } else {
            // If the key is unbalanced, we might need to fetch multiple tiles.
            // We split the key into a balanced left part and an unbalanced right part which we fetch recursively
            let (left, right) = key.split();
            tracing::debug!("Fetching balanced key: {:?}", left);
            tracing::debug!("Fetching unbalanced key: {:?}", right);
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

            tracing::debug!("Fetched unbalanced key: {:?}", key);
            left_nodes
        };

        tracing::debug!("Fetched {} nodes", nodes.len());
        Some(nodes)
    }

    #[tracing::instrument(level = "trace")]
    async fn fetch_balanced_keys(
        &self,
        key: &NodeKey,
        tree_size: u64,
    ) -> Option<Vec<(NodeKey, [u8; 32])>> {
        if let Some(value) = self.node_cache.get(key) {
            return Some(vec![(key.clone(), value)]);
        }

        let tile_id = TileId::from_node_key(key, tree_size)?;
        let tile = self.log.client.get_tile(tile_id.clone()).await;

        if tile.is_err() {
            tracing::error!("Failed to fetch tile {:?}, reason: {:?}", tile_id, tile);
        }

        let tile = tile.ok()?;
        let nodes = tile.recompute_node_keys();

        tracing::debug!("Fetched balanced key: {:?}", key);
        Some(nodes)
    }
}
