use crate::{HashOutput, ScannerImpl, log::ScannerLogInner};
use luct_client::{Client, CtClient};
use luct_core::{
    store::{Hashable, MemoryStore, StoreBase, StoreRead},
    tiling::{TileId, TilingError},
    tree::{Node, NodeKey, ProofValidationError, Tree, TreeHead},
    v1::{MerkleTreeLeaf, SignedCertificateTimestamp, SignedTreeHead},
};
use luct_store::LruCacheStore;
use std::{
    fmt::{self, Debug},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

pub(crate) struct TileFetcher<S: ScannerImpl>(
    #[allow(clippy::type_complexity)]
    Tree<LruCacheStore<TileFetchStore<S::Client>>, MemoryStore<u64, SignedCertificateTimestamp>>,
);

impl<S: ScannerImpl> Debug for TileFetcher<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TileFetcher").field(&self.0).finish()
    }
}

impl<S: ScannerImpl> TileFetcher<S> {
    pub(crate) fn new(log: &Arc<ScannerLogInner<S>>) -> Self {
        Self(Tree::new(
            // TODO: Make caps configurable
            LruCacheStore::new(TileFetchStore::new(log), 1000),
            MemoryStore::default(),
        ))
    }
}

impl<S: ScannerImpl> TileFetcher<S> {
    #[tracing::instrument(level = "trace")]
    pub(crate) async fn check_sct_inclusion(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &SignedTreeHead,
        leaf: &MerkleTreeLeaf,
    ) -> Result<u64, TilingError> {
        let Some(leaf_index) = sct.leaf_index() else {
            return Err(TilingError::LeafIndexMissing);
        };

        let tree_head = TreeHead::from(sth);

        tracing::debug!(
            "Fetching audit proof for leaf index {:?} for tree size {}",
            leaf_index,
            tree_head.tree_size()
        );

        // Need to set the sth correctly for the async proof to work
        self.0.nodes().set_tree_size(tree_head.tree_size());

        let audit_proof = self
            .0
            .get_audit_proof(&tree_head, *leaf_index)
            .await
            .map_err(TilingError::AuditProofGenerationError)?;

        audit_proof
            .validate(&tree_head, leaf)
            .map_err(TilingError::AuditProofError)?;

        Ok(audit_proof.index())
    }

    pub(crate) async fn check_sth_consistency(
        &self,
        old_sth: &SignedTreeHead,
        new_sth: &SignedTreeHead,
    ) -> Result<(), TilingError> {
        // TODO: Move these checks into TreeHead and use here as well as in consistency validation function
        if old_sth.tree_size() > new_sth.tree_size() {
            return Err(TilingError::ConsistencyProofError(
                ProofValidationError::InvalidTreeSize {
                    expected: old_sth.tree_size(),
                    received: new_sth.tree_size(),
                },
            ));
        }

        if old_sth.tree_size() == new_sth.tree_size() {
            if old_sth.sha256_root_hash() == new_sth.sha256_root_hash() {
                return Ok(());
            } else {
                return Err(TilingError::ConsistencyProofError(
                    ProofValidationError::HashMismatch,
                ));
            }
        }

        let old_tree_head = TreeHead::from(old_sth);
        let new_tree_head = TreeHead::from(new_sth);

        tracing::debug!(
            "Fetching extension proof from tree size {} to {}",
            old_tree_head.tree_size(),
            new_tree_head.tree_size()
        );

        // Need to set the sth correctly for the async proof to work
        self.0.nodes().set_tree_size(new_tree_head.tree_size());

        let consistency_proof = self
            .0
            .get_consistency_proof(&old_tree_head, &new_tree_head)
            .await
            .map_err(TilingError::ConsistencyProofGenerationError)?;

        consistency_proof
            .validate(&old_tree_head, &new_tree_head)
            .map_err(TilingError::ConsistencyProofError)?;

        Ok(())
    }
}

pub(crate) struct TileFetchStore<C> {
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
    fn new<S>(log: &ScannerLogInner<S>) -> TileFetchStore<S::Client>
    where
        C: Clone,
        S: ScannerImpl<Client = C>,
    {
        TileFetchStore {
            name: log.name.clone(),
            client: log.client.clone(),
            tree_size: AtomicU64::new(0),
        }
    }
}
