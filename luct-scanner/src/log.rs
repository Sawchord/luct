use luct_client::CtClient;
use luct_core::{store::Store, v1::SignedTreeHead};

pub(crate) struct ScannerLog<C> {
    name: String,
    client: CtClient<C>,
    sht_store: Box<dyn Store<u64, SignedTreeHead>>,
    // TODO: Supported root fingerprints
}
