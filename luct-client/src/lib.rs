use luct_core::CtLog;
use std::sync::Arc;
use thiserror::Error;
use url::Url;

pub struct CtLogClient<C> {
    log: CtLog,
    client: C,
}

pub trait Client {
    fn get(&self, url: &Url, params: &(&str, &str)) -> Result<String, ClientError>;

    // TODO: Post
}

pub struct DynClient(Arc<dyn Client>);

impl Client for DynClient {
    fn get(&self, url: &Url, params: &(&str, &str)) -> Result<String, ClientError> {
        self.0.get(url, params)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ClientError {}
