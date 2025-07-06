use crate::store::Store;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::marker::PhantomData;

// TODO: Implement a custom digest trait and make all types in this module generic on it
type HashOutput = [u8; 32];

pub trait Hashable {
    fn hash(&self) -> HashOutput;
}

#[derive(Debug, Clone)]
pub struct Tree<N, L, V> {
    nodes: N,
    leafs: L,
    values: PhantomData<V>,
}

impl<N: Store<NodeKey, HashOutput>, L: Store<u64, V>, V: Hashable> Tree<N, L, V> {
    pub fn new(node_store: N, leaf_store: L) -> Self {
        Self {
            nodes: node_store,
            leafs: leaf_store,
            values: PhantomData,
        }
    }

    pub fn insert_entry(&self, entry: V) {
        let idx = self.leafs.len() as u64;
        let old_hash = self.nodes.insert(NodeKey::leaf(idx), entry.hash());
        let old_leaf = self.leafs.insert(idx, entry);

        // FIXME: We should handle this gracefully somehow
        // Is this possible without introducing a transactional store trait?
        if old_hash.is_some() || old_leaf.is_some() {
            panic!("Inserting can only be done by one thread");
        };

        // Already update intermediate nodes, if they are power of twos
        let mut idx_mod = 2;
        while (idx + 1) % idx_mod == 0 {
            let start = idx % idx_mod;

            let key = NodeKey { start, end: idx };
            let (left, right) = key.split();
            let node = Node {
                left: self.nodes.get(&left).unwrap(),
                right: self.nodes.get(&right).unwrap(),
            };

            self.nodes.insert(key, node.hash());
            idx_mod <<= 1;
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

    pub fn get_audit_proof(&self, head: &TreeHead, index: u64) -> Option<AuditProof> {
        todo!()
    }

    pub fn get_consistency_proof(
        &self,
        old_head: &TreeHead,
        new_head: &TreeHead,
    ) -> Option<ConsistencyProof> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AuditProof(u64, Vec<HashOutput>);

impl AuditProof {
    pub fn validate(head: &TreeHead) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConsistencyProof(Vec<HashOutput>);

impl ConsistencyProof {
    pub fn validate(old_head: &TreeHead, new_head: &TreeHead) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TreeHead {
    tree_size: u64,
    head: HashOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub struct NodeKey {
    start: u64,
    end: u64,
}

impl NodeKey {
    fn leaf(idx: u64) -> Self {
        Self {
            start: idx,
            end: idx + 1,
        }
    }

    fn full_range(end: u64) -> Self {
        Self { start: 0, end }
    }

    fn split(&self) -> (Self, Self) {
        let diff = self.end - self.start;
        let split = diff.next_power_of_two() >> 1;
        let split = self.start + split;
        (
            Self {
                start: self.start,
                end: split,
            },
            Self {
                start: split,
                end: self.end,
            },
        )
    }

    fn merge(&self, other: &Self) -> Option<Self> {
        if self.end == other.start {
            Some(Self {
                start: self.start,
                end: other.end,
            })
        } else {
            None
        }
    }

    fn is_balanced(&self) -> bool {
        let diff = self.end - self.start;
        diff.is_power_of_two()
    }
}

impl PartialOrd for NodeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.end.cmp(&other.end) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.start.cmp(&other.start)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    left: HashOutput,
    right: HashOutput,
}

impl Hashable for Node {
    fn hash(&self) -> HashOutput {
        let mut hash = Sha256::new();
        hash.update([1]);
        hash.update(self.left);
        hash.update(self.right);
        hash.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    #[test]
    fn compute_tree_heads() {
        let tree = Tree::<_, _, String>::new(MemoryStore::default(), MemoryStore::default());

        tree.insert_entry("A".to_string());
        tree.insert_entry("B".to_string());
        tree.insert_entry("C".to_string());

        todo!()
    }

    impl Hashable for String {
        fn hash(&self) -> HashOutput {
            Sha256::digest(self.as_bytes()).into()
        }
    }
}
