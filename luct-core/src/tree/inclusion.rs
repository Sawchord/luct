use futures::future::join_all;

use crate::{
    store::{AsyncStore, Hashable, Store},
    tree::{HashOutput, Node, NodeKey, Tree, TreeHead},
};

impl<N, L, V> Tree<N, L, V>
where
    N: Store<NodeKey, HashOutput>,
    V: Hashable,
{
    /// This follows RFC 9162 2.1.3.1
    pub fn get_audit_proof(&self, head: &TreeHead, index: u64) -> Option<AuditProof> {
        if index >= head.tree_size {
            return None;
        }

        let path = get_audit_proof(head, index, |key| self.nodes.get(&key));
        let mut path = path.into_iter().collect::<Option<Vec<_>>>()?;

        path.reverse();
        Some(AuditProof { index, path })
    }
}

impl<N, L, V> Tree<N, L, V>
where
    N: AsyncStore<NodeKey, HashOutput>,
    V: Hashable,
{
    pub async fn get_audit_proof_async(&self, head: &TreeHead, index: u64) -> Option<AuditProof> {
        if index >= head.tree_size {
            return None;
        }

        let path = get_audit_proof(head, index, |key| self.nodes.get(key));
        let path = join_all(path).await;
        let mut path = path.into_iter().collect::<Option<Vec<_>>>()?;

        path.reverse();
        Some(AuditProof { index, path })
    }
}

fn get_audit_proof<F, O>(head: &TreeHead, index: u64, get: F) -> Vec<O>
where
    F: Fn(NodeKey) -> O,
{
    let mut n = NodeKey::full_range(head.tree_size);
    let m = index;

    let mut path = vec![];

    while !n.is_leaf() {
        let (left, right) = n.split();
        if m < right.start {
            let elem = get(right);
            path.push(elem);

            n = left;
        } else {
            let elem = get(left);
            path.push(elem);

            n = right;
        }
    }

    path
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

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
}
