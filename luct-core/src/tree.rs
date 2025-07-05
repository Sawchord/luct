use crate::store::Store;
use sha2::{Digest, Sha256};
use std::marker::PhantomData;

type HashOutput = [u8; 32];

pub(crate) trait Hashable {
    fn hash(&self) -> HashOutput;
}

#[derive(Debug, Clone)]
pub(crate) struct Tree<N, L, V> {
    nodes: N,
    leafs: L,
    values: PhantomData<V>,
}

impl<N: Store<NodeKey, HashOutput>, L: Store<u64, V>, V: Hashable> Tree<N, L, V> {
    pub fn insert_entry(&self, entry: V) {
        let idx = self.leafs.len() as u64;
        let old_hash = self.nodes.insert(NodeKey::full(idx + 1), entry.hash());
        let old_leaf = self.leafs.insert(idx, entry);

        // FIXME: We should handle this gracefully somehow
        // Is this possible without introducing a transactional store trait?
        if old_hash.is_some() || old_leaf.is_some() {
            panic!("Inserting can only be done by one thread");
        };

        // Already update intermediate nodes, if they are power of twos
        let mut idx_mod = 2;
        while idx % idx_mod == 0 {
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
        todo!()
    }

    pub fn get_tree_head(&self) -> Option<TreeHead> {
        let idx = self.leafs.len() as u64;
        self.nodes
            .get(&NodeKey { start: 0, end: idx })
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
pub(crate) struct AuditProof(Vec<HashOutput>);

impl AuditProof {
    pub fn validate(head: &TreeHead) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ConsistencyProof(Vec<HashOutput>);

impl ConsistencyProof {
    pub fn validate(old_head: &TreeHead, new_head: &TreeHead) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TreeHead {
    tree_size: u64,
    head: HashOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]

struct NodeKey {
    start: u64,
    end: u64,
}

impl NodeKey {
    fn full(end: u64) -> Self {
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
        hash.update(&[1]);
        hash.update(&self.left);
        hash.update(&self.right);
        hash.finalize().into()
    }
}
