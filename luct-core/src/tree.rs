use crate::store::{Hashable, Store};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{cmp::Ordering, marker::PhantomData};

// TODO: Implement a custom digest trait and make all types in this module generic on it
pub(crate) type HashOutput = [u8; 32];

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
        let entry_key = NodeKey::leaf(idx);
        let old_hash = self.nodes.insert(entry_key, entry.hash());
        let old_leaf = self.leafs.insert(idx, entry);

        // FIXME: We should handle this gracefully somehow
        // Is this possible without introducing a transactional store trait?
        if old_hash.is_some() || old_leaf.is_some() {
            panic!("Inserting can only be done by one thread");
        };

        // Already update intermediate nodes, if they are power of twos
        let end = idx + 1;
        let mut diff = 2;

        while end % diff == 0 {
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

    /// This follows RFC 9162 2.1.3.1
    pub fn get_audit_proof(&self, head: &TreeHead, index: u64) -> Option<AuditProof> {
        if index >= head.tree_size {
            return None;
        }

        let mut n = NodeKey::full_range(head.tree_size);
        let m = index;

        let mut path = vec![];

        while n.start + 1 != n.end {
            let (left, right) = n.split();
            if m < right.start {
                let elem = self.nodes.get(&right)?;
                path.push(elem);

                n = left;
            } else {
                let elem = self.nodes.get(&left)?;
                path.push(elem);

                n = right;
            }
        }

        path.reverse();
        Some(AuditProof { index, path })
    }

    /// This follows RFC 9162 2.1.4.1
    pub fn get_consistency_proof(
        &self,
        first: &TreeHead,
        second: &TreeHead,
    ) -> Option<ConsistencyProof> {
        if first.tree_size > second.tree_size {
            return None;
        }

        let tree_size = second.tree_size;

        let mut n = NodeKey::full_range(tree_size);
        let mut m = first.tree_size;
        let mut known = true;

        let mut path = vec![];

        while m + n.start != n.end {
            let (left, right) = n.split();
            if m <= right.start {
                let elem = self.nodes.get(&right)?;
                path.push(elem);
                n = left;
            } else {
                let elem = self.nodes.get(&left)?;
                path.push(elem);

                known = false;
                m -= right.start;
                n = right;
            }
        }

        if !known {
            let elem = self.nodes.get(&n)?;
            path.push(elem);
        }

        path.reverse();
        Some(ConsistencyProof { path })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AuditProof {
    pub(crate) index: u64,
    pub(crate) path: Vec<HashOutput>,
}

impl AuditProof {
    pub fn validate(&self, head: &TreeHead, leaf: &impl Hashable) -> bool {
        if head.tree_size < self.index {
            return false;
        }

        let mut f_n = self.index;
        let mut s_n = head.tree_size - 1;
        let mut r = leaf.hash();

        for p in &self.path {
            if s_n == 0 {
                return false;
            }

            if f_n & 1 == 1 || f_n == s_n {
                r = Node { left: *p, right: r }.hash();

                while f_n & 1 != 1 && f_n != 0 {
                    f_n >>= 1;
                    s_n >>= 1;
                }
            } else {
                r = Node { left: r, right: *p }.hash();
            }

            f_n >>= 1;
            s_n >>= 1;
        }

        r == head.head && s_n == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConsistencyProof {
    pub(crate) path: Vec<HashOutput>,
}

impl ConsistencyProof {
    /// This follows RFC 9162 2.1.4.2
    pub fn validate(&self, first: &TreeHead, second: &TreeHead) -> bool {
        if first.tree_size > second.tree_size {
            return false;
        };
        if first == second && self.path.is_empty() {
            return true;
        }

        let path: Vec<&HashOutput> = if first.tree_size.is_power_of_two() {
            std::iter::once(&first.head)
                .chain(self.path.iter())
                .collect()
        } else {
            self.path.iter().collect()
        };

        let mut f_n = first.tree_size - 1;
        let mut s_n = second.tree_size - 1;

        while f_n & 1 == 1 {
            f_n >>= 1;
            s_n >>= 1;
        }

        let mut f_r = *path[0];
        let mut s_r = *path[0];

        for &c in &path[1..] {
            if s_n == 0 {
                return false;
            }

            if f_n & 1 == 1 || f_n == s_n {
                f_r = Node {
                    left: *c,
                    right: f_r,
                }
                .hash();

                s_r = Node {
                    left: *c,
                    right: s_r,
                }
                .hash();

                while f_n & 1 == 0 && f_n != 0 {
                    f_n >>= 1;
                    s_n >>= 1;
                }
            } else {
                s_r = Node {
                    left: s_r,
                    right: *c,
                }
                .hash();
            }

            f_n >>= 1;
            s_n >>= 1;
        }

        f_r == first.head && s_r == second.head && s_n == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TreeHead {
    pub(crate) tree_size: u64,
    pub(crate) head: HashOutput,
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

    fn split_idx(&self) -> u64 {
        let diff = self.end - self.start;
        diff.next_power_of_two() >> 1
    }

    fn split(&self) -> (Self, Self) {
        let split = self.split_idx();
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.end.cmp(&other.end) {
            Ordering::Equal => {}
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
    fn compute_inclusion_proofs() {
        let tree = Tree::<_, _, String>::new(MemoryStore::default(), MemoryStore::default());

        tree.insert_entry("A".to_string());
        tree.insert_entry("B".to_string());
        tree.insert_entry("C".to_string());

        // Generate tree head
        let tree_head1 = tree.recompute_tree_head();

        tree.insert_entry("D".to_string());
        let tree_head2 = tree.recompute_tree_head();

        tree.insert_entry("E".to_string());
        tree.insert_entry("F".to_string());
        let tree_head3 = tree.recompute_tree_head();

        tree.insert_entry("G".to_string());
        let tree_head4 = tree.recompute_tree_head();

        tree.insert_entry("H".to_string());

        let proof1 = tree
            .get_consistency_proof(&tree_head1, &tree_head4)
            .unwrap();
        assert_eq!(proof1.path.len(), 4);
        assert!(proof1.validate(&tree_head1, &tree_head4));

        let proof2 = tree
            .get_consistency_proof(&tree_head2, &tree_head4)
            .unwrap();
        assert_eq!(proof2.path.len(), 1);
        assert_eq!(proof1.path[3], proof2.path[0]);
        assert!(proof2.validate(&tree_head2, &tree_head4));

        let proof3 = tree
            .get_consistency_proof(&tree_head3, &tree_head4)
            .unwrap();
        assert_eq!(proof3.path.len(), 3);
        assert!(proof3.validate(&tree_head3, &tree_head4));

        let proof4 = tree
            .get_consistency_proof(&tree_head4, &tree_head4)
            .unwrap();
        assert!(proof4.path.is_empty());
        assert!(proof4.validate(&tree_head4, &tree_head4));
    }

    #[test]
    fn compute_audit_proofs() {
        let tree = Tree::<_, _, String>::new(MemoryStore::default(), MemoryStore::default());

        tree.insert_entry("A".to_string());
        tree.insert_entry("B".to_string());
        tree.insert_entry("C".to_string());
        tree.insert_entry("D".to_string());
        tree.insert_entry("E".to_string());
        tree.insert_entry("F".to_string());
        tree.insert_entry("G".to_string());

        let head = tree.recompute_tree_head();

        let proof1 = tree.get_audit_proof(&head, 0).unwrap();
        assert_eq!(proof1.path.len(), 3);
        assert!(proof1.validate(&head, &"A".to_string()));

        let proof2 = tree.get_audit_proof(&head, 3).unwrap();
        assert_eq!(proof2.path.len(), 3);
        assert!(proof2.validate(&head, &"D".to_string()));

        let proof3 = tree.get_audit_proof(&head, 4).unwrap();
        assert_eq!(proof3.path.len(), 3);
        assert!(proof3.validate(&head, &"E".to_string()));

        let proof4 = tree.get_audit_proof(&head, 6).unwrap();
        assert_eq!(proof4.path.len(), 2);
        assert!(proof4.validate(&head, &"G".to_string()));
    }

    impl Hashable for String {
        fn hash(&self) -> HashOutput {
            Sha256::digest(self.as_bytes()).into()
        }
    }
}
