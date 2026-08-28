use crate::store::{MemoryStore, StoreBase, StoreRead, StoreWrite};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::*;
use rand::{Rng, RngExt, SeedableRng, rngs::ChaCha8Rng};
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

#[derive(Debug, Clone, Default)]
struct TreeTestStore(Rc<RefCell<HashMap<NodeKey, HashOutput>>>);

impl TreeTestStore {
    fn reverse_lookup(&self, hash: &HashOutput) -> NodeKey {
        self.0
            .borrow()
            .iter()
            .find(|(_, value)| value == &hash)
            .unwrap()
            .0
            .clone()
    }

    fn keys(&self) -> Vec<NodeKey> {
        self.0.borrow().keys().cloned().collect()
    }
}

impl StoreBase for TreeTestStore {
    type Key = NodeKey;
    type Value = HashOutput;
}

impl StoreRead for TreeTestStore {
    async fn get(&self, key: Self::Key) -> Option<Self::Value> {
        self.0.borrow().get(&key).cloned()
    }

    async fn len(&self) -> usize {
        self.0.borrow().len()
    }
}

impl StoreWrite for TreeTestStore {
    async fn insert(&self, key: Self::Key, value: Self::Value) {
        self.0.borrow_mut().insert(key, value);
    }

    async fn delete(&self, _key: Self::Key) -> bool {
        unimplemented!()
    }
}

async fn fill_tree_until<L>(tree: &Tree<TreeTestStore, L>, rng: &mut dyn Rng, target_size: u64)
where
    L: StoreBase<Key = u64, Value = HashOutput> + AppendableStore,
{
    let start = tree
        .get_latest_tree_head()
        .await
        .map(|head| head.tree_size())
        .unwrap_or(0);

    for _ in start..target_size {
        let hash: HashOutput = rng.random();
        tree.insert_entry(hash).await;
    }
}

fn lookup_proof(proof: &ConsistencyProof, store: &TreeTestStore) -> Vec<NodeKey> {
    proof
        .path
        .iter()
        .map(|hash| store.reverse_lookup(hash))
        .collect()
}

#[tokio::test]
async fn proof_cut_test() {
    let mut rng = ChaCha8Rng::seed_from_u64(6767);

    let test_store = TreeTestStore::default();
    let tree = Tree::<TreeTestStore, MemoryStore<u64, HashOutput>>::new(
        test_store.clone(),
        MemoryStore::default(),
    );

    let a = 842;
    let b = 2865;
    let c = 3614;
    let d = 4000;

    fill_tree_until(&tree, &mut rng, a).await;
    let head_a = tree.recompute_tree_head().await;

    fill_tree_until(&tree, &mut rng, b).await;
    let head_b = tree.recompute_tree_head().await;

    fill_tree_until(&tree, &mut rng, c).await;
    let head_c = tree.recompute_tree_head().await;

    fill_tree_until(&tree, &mut rng, d).await;
    let head_d = tree.recompute_tree_head().await;

    let proof_a_c = tree.get_consistency_proof(&head_a, &head_c).await.unwrap();
    proof_a_c.validate(&head_a, &head_c).unwrap();
    println!("(a-c): {:?}", lookup_proof(&proof_a_c, &test_store));

    let proof_b_d = tree.get_consistency_proof(&head_b, &head_d).await.unwrap();
    proof_b_d.validate(&head_b, &head_d).unwrap();
    println!("(b-d): {:?}", lookup_proof(&proof_b_d, &test_store));

    let proof_a_b = tree.get_consistency_proof(&head_a, &head_b).await.unwrap();
    proof_a_b.validate(&head_a, &head_b).unwrap();
    println!("(a-b): {:?}", lookup_proof(&proof_a_b, &test_store));

    let proof_b_c = tree.get_consistency_proof(&head_b, &head_c).await.unwrap();
    proof_b_c.validate(&head_b, &head_c).unwrap();
    println!("(b-c): {:?}", lookup_proof(&proof_b_c, &test_store));

    let proof_c_d = tree.get_consistency_proof(&head_c, &head_d).await.unwrap();
    proof_c_d.validate(&head_c, &head_d).unwrap();
    println!("(c-d): {:?}", lookup_proof(&proof_c_d, &test_store));

    todo!()
}
