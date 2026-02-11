use crate::store::{Hashable, IndexedStore, Store};
pub use crate::tree::{
    consistency::ConsistencyProof,
    inclusion::AuditProof,
    node::{Node, NodeKey},
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use thiserror::Error;

mod consistency;
mod inclusion;
mod node;

pub(crate) type HashOutput = [u8; 32];

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ProofGenerationError {
    #[error("Index {index} not found in tree of size {tree_size}")]
    InvalidIndex { tree_size: u64, index: u64 },

    #[error("Tree of size {small_tree_size} is smaller than {large_tree_size}")]
    InvalidTreeSize {
        small_tree_size: u64,
        large_tree_size: u64,
    },

    #[error("Failed to fetch key {0:?} from the store")]
    KeyNotFound(NodeKey),
}

#[derive(Debug, Clone)]
pub struct Tree<N, L, V> {
    nodes: N,
    leafs: L,
    values: PhantomData<V>,
}

impl<N, L, V> Tree<N, L, V> {
    pub fn new(node_store: N, leaf_store: L) -> Self {
        Self {
            nodes: node_store,
            leafs: leaf_store,
            values: PhantomData,
        }
    }

    pub fn nodes(&self) -> &N {
        &self.nodes
    }
}

impl<N, L, V> Tree<N, L, V>
where
    N: Store<NodeKey, HashOutput>,
    L: IndexedStore<V>,
    V: Hashable,
{
    pub fn insert_entry(&self, entry: V) {
        let entry_hash = entry.hash();
        let idx = self.leafs.insert_indexed(entry);
        let entry_key = NodeKey::leaf(idx);
        self.nodes.insert(entry_key, entry_hash);

        // Already update intermediate nodes, if they are power of twos
        let end = idx + 1;
        let mut diff = 2;

        while end.is_multiple_of(diff) {
            let start = end - diff;

            let key = NodeKey { start, end };
            let (left, right) = key.split();

            let node = Node {
                left: self.nodes.get(&left).unwrap(),
                right: self.nodes.get(&right).unwrap(),
            };

            self.nodes.insert(key, node.hash());

            diff <<= 1;
        }
    }

    pub fn recompute_tree_head(&self) -> TreeHead {
        let tree_size = self.leafs.len() as u64;
        let mut current_key = NodeKey::full_range(tree_size);
        let mut balanced_nodes = vec![];

        while !current_key.is_balanced() {
            let (left, right) = current_key.split();
            assert!(left.is_balanced());
            balanced_nodes.push(left);
            current_key = right;
        }

        let mut current_node_hash = self.nodes.get(&current_key).unwrap();
        while let Some(left_key) = balanced_nodes.pop() {
            let current_node = Node {
                left: self.nodes.get(&left_key).unwrap(),
                right: self.nodes.get(&current_key).unwrap(),
            };

            current_key = left_key.merge(&current_key).unwrap();
            current_node_hash = current_node.hash();
            self.nodes.insert(current_key.clone(), current_node_hash);
        }

        TreeHead {
            tree_size,
            head: current_node_hash,
        }
    }

    pub fn get_latest_tree_head(&self) -> Option<TreeHead> {
        let idx = self.leafs.len() as u64;
        self.nodes
            .get(&NodeKey::full_range(idx))
            .map(|head| TreeHead {
                tree_size: idx,
                head,
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TreeHead {
    pub(crate) tree_size: u64,
    pub(crate) head: HashOutput,
}

impl TreeHead {
    pub fn tree_size(&self) -> u64 {
        self.tree_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    impl Hashable for String {
        fn hash(&self) -> HashOutput {
            Sha256::digest(self.as_bytes()).into()
        }
    }

    impl Hashable for HashOutput {
        fn hash(&self) -> HashOutput {
            Sha256::digest(self).into()
        }
    }
}
