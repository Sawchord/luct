use luct_core::{
    store::MemoryStore,
    tiling::{IsTileFetchStore, TilingError},
    tree::{ProofValidationError, Tree, TreeHead},
    v1::{MerkleTreeLeaf, SignedCertificateTimestamp, SignedTreeHead},
};
use std::fmt::{self, Debug};

pub struct TileFetcher<SCT, STH> {
    sct_fetcher: Tree<SCT, MemoryStore<u64, SignedCertificateTimestamp>>,
    sth_fetcher: Tree<STH, MemoryStore<u64, SignedCertificateTimestamp>>,
}

impl<SCT: Debug, STH: Debug> Debug for TileFetcher<SCT, STH> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileFetcher")
            .field("sct_fetcher", &self.sct_fetcher)
            .field("sth_fetcher", &self.sth_fetcher)
            .finish()
    }
}

impl<SCT, STH> TileFetcher<SCT, STH> {
    pub fn new(sct_store: SCT, sth_store: STH) -> Self {
        Self {
            sct_fetcher: Tree::new(sct_store, MemoryStore::default()),
            sth_fetcher: Tree::new(sth_store, MemoryStore::default()),
        }
    }
}

impl<SCT, STH> TileFetcher<SCT, STH>
where
    SCT: IsTileFetchStore,
    STH: IsTileFetchStore,
{
    pub async fn check_sct_inclusion(
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
        self.sct_fetcher
            .nodes()
            .set_tree_size(tree_head.tree_size());

        let audit_proof = self
            .sct_fetcher
            .get_audit_proof(&tree_head, *leaf_index)
            .await
            .map_err(TilingError::AuditProofGenerationError)?;

        audit_proof
            .validate(&tree_head, leaf)
            .map_err(TilingError::AuditProofError)?;

        Ok(audit_proof.index())
    }

    pub async fn check_sth_consistency(
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
        self.sth_fetcher
            .nodes()
            .set_tree_size(new_tree_head.tree_size());

        let consistency_proof = self
            .sth_fetcher
            .get_consistency_proof(&old_tree_head, &new_tree_head)
            .await
            .map_err(TilingError::ConsistencyProofGenerationError)?;

        consistency_proof
            .validate(&old_tree_head, &new_tree_head)
            .map_err(TilingError::ConsistencyProofError)?;

        Ok(())
    }
}
