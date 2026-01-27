use crate::{Client, ClientError};
use async_oneshot::{Sender, oneshot};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// Wraps an inner [`Client`] and deduplicates running requests.
///
/// The endpoint must be idempotent.
/// In particular, the following things must be guaranteed:
///
/// - The endpoint must return the same response on the same request
/// - The deduplication may fail due to TOCTOU, and sending the same request
///   twice must not change the servers behavioru
#[derive(Debug, Clone, Default)]
pub struct RequestDeduplicationClient<C> {
    inner: C,
    requests: Arc<Mutex<BTreeMap<DeduplicationKey, Vec<Sender<Response>>>>>,
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
        let response = self
            .wait_or_try_get(url, params, async {
                match self.inner.get(url, params).await {
                    Ok((status, data)) => Response::String(status, data),
                    Err(err) => Response::Error(err),
                }
            })
            .await;

        match response {
            Response::String(status, data) => Ok((status, data)),
            Response::Binary(_, _) => panic!(),
            Response::Error(client_error) => Err(client_error),
        }
    }

    async fn get_bin(
        &self,
        url: &url::Url,
        params: &[(&str, &str)],
    ) -> Result<(u16, std::sync::Arc<Vec<u8>>), ClientError> {
        let response = self
            .wait_or_try_get(url, params, async {
                match self.inner.get_bin(url, params).await {
                    Ok((status, data)) => Response::Binary(status, data),
                    Err(err) => Response::Error(err),
                }
            })
            .await;

        match response {
            Response::String(_, _) => panic!(),
            Response::Binary(status, data) => Ok((status, data)),
            Response::Error(client_error) => Err(client_error),
        }
    }
}

impl<C: Client> RequestDeduplicationClient<C> {
    async fn wait_or_try_get(
        &self,
        url: &url::Url,
        params: &[(&str, &str)],
        getter: impl Future<Output = Response>,
    ) -> Response {
        let key = DeduplicationKey {
            url: url.clone(),
            params: params
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        };

        let (rx, request) = {
            let mut requests = self.requests.lock().unwrap();

            let (tx, rx) = oneshot::<Response>();
            match requests.get_mut(&key) {
                Some(ongoing_requests) => {
                    println!("Dedup request to {}", url);
                    ongoing_requests.push(tx);

                    (rx, None)
                }
                None => {
                    println!("New request to {}", url);

                    requests.insert(key.clone(), vec![tx]);

                    let request = async move {
                        let response = getter.await;
                        let mut requests = self.requests.lock().unwrap();

                        println!(
                            "Sending response of {} to {} requesters",
                            url,
                            requests.len()
                        );
                        for mut tx in requests
                            .remove(&key)
                            .expect("Key no longer exist. This is a bug")
                        {
                            tx.send(response.clone()).unwrap();
                        }
                    };

                    (rx, Some(request))
                }
            }
        };

        // If we are making a request, wait on it
        if let Some(request) = request {
            request.await;
        }

        // Await on receiving the answer
        rx.await
            .expect("Dedup channel closed instead of answered. This is a bug")
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
