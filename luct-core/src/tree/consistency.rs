use crate::{
    store::{Hashable, IndexedStore, Store},
    tree::{HashOutput, Node, NodeKey, Tree, TreeHead},
};

impl<N, L, V> Tree<N, L, V>
where
    N: Store<NodeKey, HashOutput>,
    L: IndexedStore<V>,
    V: Hashable,
{
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

#[cfg(test)]
mod tests {
    use crate::store::MemoryStore;

    use super::*;

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
}
