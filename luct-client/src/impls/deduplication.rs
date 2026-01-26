use crate::{Client, ClientError};
use async_oneshot::Receiver;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// Wraps an inner [`Client`] and deduplicates running requests.
///
/// The endpoint must be idempotent.
/// In particular, the following things must be guarenteed:
///
/// - The endpoint must return the same response on the same request
/// - The deduplication may fail due to TOCTOU, and sending the same request
///   twice must not change the servers behavioru
#[derive(Debug, Clone, Default)]
pub struct RequestDeduplicationClient<C> {
    inner: C,
    requests: Arc<Mutex<BTreeMap<DeduplicationKey, Vec<Receiver<Response>>>>>,
}

impl<C> RequestDeduplicationClient<C> {
    pub fn new(inner: C) -> Self {
        Self {
            inner,
            requests: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl<C: Client> Client for RequestDeduplicationClient<C> {
    async fn get(
        &self,
        url: &url::Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, std::sync::Arc<String>), ClientError> {
        todo!()
    }

    async fn get_bin(
        &self,
        url: &url::Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, std::sync::Arc<Vec<u8>>), ClientError> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DeduplicationKey {
    url: url::Url,
    params: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
enum Response {
    String(u16, std::sync::Arc<String>),
    Binary(u16, std::sync::Arc<Vec<u8>>),
    Error(ClientError),
}
