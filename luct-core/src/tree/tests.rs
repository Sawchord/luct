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

    fill_tree_until(&tree, &mut rng, 5).await;
    let head_5 = tree.recompute_tree_head().await;

    fill_tree_until(&tree, &mut rng, 17).await;
    let head_17 = tree.recompute_tree_head().await;

    fill_tree_until(&tree, &mut rng, 27).await;
    let head_27 = tree.recompute_tree_head().await;

    fill_tree_until(&tree, &mut rng, 31).await;
    let head_31 = tree.recompute_tree_head().await;

    let proof_5_27 = tree.get_consistency_proof(&head_5, &head_27).await.unwrap();
    proof_5_27.validate(&head_5, &head_27).unwrap();
    println!("{:?}", lookup_proof(&proof_5_27, &test_store));

    let proof_17_31 = tree
        .get_consistency_proof(&head_17, &head_31)
        .await
        .unwrap();
    proof_17_31.validate(&head_17, &head_31).unwrap();
    println!("{:?}", lookup_proof(&proof_17_31, &test_store));

    let proof_5_17 = tree.get_consistency_proof(&head_5, &head_17).await.unwrap();
    proof_5_17.validate(&head_5, &head_17).unwrap();
    println!("{:?}", lookup_proof(&proof_5_17, &test_store));

    let proof_17_27 = tree
        .get_consistency_proof(&head_17, &head_27)
        .await
        .unwrap();
    proof_17_27.validate(&head_17, &head_27).unwrap();
    println!("{:?}", lookup_proof(&proof_17_27, &test_store));

    let proof_27_31 = tree
        .get_consistency_proof(&head_27, &head_31)
        .await
        .unwrap();
    proof_27_31.validate(&head_27, &head_31).unwrap();
    println!("{:?}", lookup_proof(&proof_27_31, &test_store));

    todo!()
}
