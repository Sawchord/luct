// TODO: Implement Filestore

use luct_core::store::{OrderedStore, Store};
use std::{
    marker::PhantomData,
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

pub trait FilesystemStoreKey: Clone + Ord {
    fn serialize_key(&self) -> String;
    fn deserialize_key(key: &str) -> Option<Self>;
}

pub trait FilesystemStoreValue: Clone {
    fn serialize_value(&self) -> String;
    fn deserialize_value(value: &str) -> Option<Self>;
}

#[derive(Clone)]
pub struct FilesystemStore<K, V> {
    _kv: PhantomData<(K, V)>,
    _path: PathBuf,
    tx: Sender<StoreRequest<K, V>>,
}

impl<K: FilesystemStoreKey, V: FilesystemStoreValue> FilesystemStore<K, V> {
    pub fn new(path: PathBuf) -> FilesystemStore<K, V> {
        let (tx, rx) = channel();
        start_storage_loop(rx, path.clone());

        Self {
            _kv: PhantomData,
            _path: path,
            tx,
        }
    }
}

impl<K: FilesystemStoreKey, V: FilesystemStoreValue> Store<K, V> for FilesystemStore<K, V> {
    fn insert(&self, key: K, value: V) {
        let answer = Answer::new();
        self.tx
            .send(StoreRequest::Insert {
                key,
                value,
                answer: answer.clone(),
            })
            .unwrap();
        answer.await_answer()
    }

    fn get(&self, key: &K) -> Option<V> {
        let answer = Answer::new();
        self.tx
            .send(StoreRequest::Get {
                key: key.clone(),
                answer: answer.clone(),
            })
            .unwrap();
        answer.await_answer()
    }

    fn len(&self) -> usize {
        let answer = Answer::new();
        self.tx.send(StoreRequest::Len(answer.clone())).unwrap();
        answer.await_answer()
    }
}

impl<K: FilesystemStoreKey, V: FilesystemStoreValue> OrderedStore<K, V> for FilesystemStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        let answer = Answer::new();
        self.tx.send(StoreRequest::Last(answer.clone())).unwrap();
        answer.await_answer()
    }
}

fn start_storage_loop<K: FilesystemStoreKey, V: FilesystemStoreValue>(
    rx: Receiver<StoreRequest<K, V>>,
    path: PathBuf,
) {
    loop {
        match rx.recv() {
            Ok(StoreRequest::Get { key, answer }) => todo!(),
            Ok(StoreRequest::Insert { key, value, answer }) => todo!(),
            Ok(StoreRequest::Last(answer)) => todo!(),
            Ok(StoreRequest::Len(answer)) => todo!(),
            Err(_) => break,
        }
    }
    todo!()
}

enum StoreRequest<K, V> {
    Get {
        key: K,
        answer: Answer<Option<V>>,
    },
    Insert {
        key: K,
        value: V,
        answer: Answer<()>,
    },
    Len(Answer<usize>),
    Last(Answer<Option<(K, V)>>),
}

#[derive(Clone)]
struct Answer<V> {
    response: Arc<Mutex<Option<V>>>,
    done: Arc<Condvar>,
}

impl<V> Answer<V> {
    fn new() -> Self {
        Self {
            response: Arc::new(Mutex::new(None)),
            done: Arc::new(Condvar::new()),
        }
    }

    fn await_answer(&self) -> V {
        let mut lock = self.response.lock().unwrap();
        loop {
            if let Some(val) = lock.take() {
                return val;
            } else {
                lock = self.done.wait(lock).unwrap();
            };
        }
    }

    fn answer(self, value: V) {
        *self.response.lock().unwrap() = Some(value);
        self.done.notify_all();
    }
}
