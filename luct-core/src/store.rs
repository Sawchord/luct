use crate::tree::HashOutput;

mod r#async;
mod memory;

pub use crate::store::r#async::{
    AsyncAppendableStore, AsyncOrderedStore, AsyncOrderedStoreRead, AsyncSearchableStore,
    AsyncSearchableStoreRead, AsyncStore, AsyncStoreRead, AsyncStoreWrite,
};
pub use crate::store::memory::MemoryStore;

/// Trait indicating that an object can be hased with respect to the CT protocol
///
/// This for now always refers to the Sha256 algorithm, but this might change in the future
pub trait Hashable {
    /// Hash the object
    fn hash(&self) -> HashOutput;
}

pub trait StoreBase {
    type Key;
    type Value;
}
