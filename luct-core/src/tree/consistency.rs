use crate::{
    store::{AsyncStore, Hashable, Store},
    tree::{HashOutput, Node, NodeKey, ProofGenerationError, ProofValidationError, Tree, TreeHead},
};
use futures::{FutureExt, future::join_all};

impl<N, L, V> Tree<N, L, V>
where
    N: Store<NodeKey, HashOutput>,
    V: Hashable,
{
    /// This follows RFC 9162 2.1.4.1
    pub fn get_consistency_proof(
        &self,
        first: &TreeHead,
        second: &TreeHead,
    ) -> Result<ConsistencyProof, ProofGenerationError> {
        if first.tree_size > second.tree_size {
            return Err(ProofGenerationError::InvalidTreeSize {
                expected: first.tree_size,
                received: second.tree_size,
            });
        }

        let path = get_consistency_proof(first, second, |key| {
            self.nodes
                .get(&key)
                .ok_or(ProofGenerationError::KeyNotFound(key))
        });
        let mut path = path
            .into_iter()
            .collect::<Result<Vec<HashOutput>, ProofGenerationError>>()?;

        path.reverse();
        Ok(ConsistencyProof { path })
    }
}

impl<N, L, V> Tree<N, L, V>
where
    N: AsyncStore<NodeKey, HashOutput>,
    V: Hashable,
{
    pub async fn get_consistency_proof_async(
        &self,
        first: &TreeHead,
        second: &TreeHead,
    ) -> Result<ConsistencyProof, ProofGenerationError> {
        if first.tree_size >= second.tree_size {
            return Err(ProofGenerationError::InvalidTreeSize {
                expected: first.tree_size,
                received: second.tree_size,
            });
        }

        let path = get_consistency_proof(first, second, |key| {
            self.nodes
                .get(key.clone())
                .map(|result| result.ok_or(ProofGenerationError::KeyNotFound(key)))
        });
        let path = join_all(path).await;
        let mut path = path
            .into_iter()
            .collect::<Result<Vec<HashOutput>, ProofGenerationError>>()?;

        path.reverse();
        Ok(ConsistencyProof { path })
    }
}

fn get_consistency_proof<F, O>(first: &TreeHead, second: &TreeHead, get: F) -> Vec<O>
where
    F: Fn(NodeKey) -> O,
{
    let mut n = NodeKey::full_range(second.tree_size);
    let m = first.tree_size;
    let mut known = true;

    let mut path = vec![];

    while m != n.end {
        let (left, right) = n.split();
        if m <= right.start {
            let elem = get(right);
            path.push(elem);
            n = left;
        } else {
            let elem = get(left);
            path.push(elem);

            known = false;
            n = right;
        }
    }

    if !known {
        let elem = get(n);
        path.push(elem);
    }

    path
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ConsistencyProof {
    pub(crate) path: Vec<HashOutput>,
}

impl ConsistencyProof {
    /// This follows RFC 9162 2.1.4.2
    pub fn validate(
        &self,
        first: &TreeHead,
        second: &TreeHead,
    ) -> Result<(), ProofValidationError> {
        if first.tree_size > second.tree_size {
            return Err(ProofValidationError::InvalidTreeSize {
                expected: first.tree_size,
                received: second.tree_size,
            });
        };
        if first == second && self.path.is_empty() {
            return Ok(());
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
                return Err(ProofValidationError::PathTooShort);
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

        if s_n != 0 {
            return Err(ProofValidationError::PathTooLong);
        }

        if f_r != first.head || s_r != second.head {
            return Err(ProofValidationError::HashMismatch);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng, rngs::ChaCha8Rng};

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
        proof1.validate(&tree_head1, &tree_head4).unwrap();

        let proof2 = tree
            .get_consistency_proof(&tree_head2, &tree_head4)
            .unwrap();
        assert_eq!(proof2.path.len(), 1);
        assert_eq!(proof1.path[3], proof2.path[0]);
        proof2.validate(&tree_head2, &tree_head4).unwrap();

        let proof3 = tree
            .get_consistency_proof(&tree_head3, &tree_head4)
            .unwrap();
        assert_eq!(proof3.path.len(), 3);
        proof3.validate(&tree_head3, &tree_head4).unwrap();

        let proof4 = tree
            .get_consistency_proof(&tree_head4, &tree_head4)
            .unwrap();
        assert!(proof4.path.is_empty());
        proof4.validate(&tree_head4, &tree_head4).unwrap();
    }

    #[test]
    fn randomized_inclusion_proof() {
        let first_size = 4973;
        let second_size = 5009;
        let mut rng = ChaCha8Rng::seed_from_u64(1337);

        let tree = Tree::<_, _, HashOutput>::new(MemoryStore::default(), MemoryStore::default());

        for _ in 0..first_size {
            let mut entry = [0; 32];
            rng.fill_bytes(&mut entry);
            tree.insert_entry(entry);
        }

        let first_th = tree.recompute_tree_head();

        for _ in first_size..second_size {
            let mut entry = [0; 32];
            rng.fill_bytes(&mut entry);
            tree.insert_entry(entry);
        }

        let second_th = tree.recompute_tree_head();

        let proof = tree.get_consistency_proof(&first_th, &second_th).unwrap();
        proof.validate(&first_th, &second_th).unwrap();
    }
}
