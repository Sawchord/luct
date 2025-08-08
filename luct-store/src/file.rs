use luct_core::store::{OrderedStore, Store};
use std::{
    fs::OpenOptions,
    io::Write,
    marker::PhantomData,
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use crate::{StringStoreKey, StringStoreValue};

#[derive(Clone)]
pub struct FilesystemStore<K, V> {
    _kv: PhantomData<(K, V)>,
    _path: PathBuf,
    tx: Sender<StoreRequest<K, V>>,
}

impl<K: StringStoreKey, V: StringStoreValue> FilesystemStore<K, V> {
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

impl<K: StringStoreKey, V: StringStoreValue> Store<K, V> for FilesystemStore<K, V> {
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

impl<K: StringStoreKey, V: StringStoreValue> OrderedStore<K, V> for FilesystemStore<K, V> {
    fn last(&self) -> Option<(K, V)> {
        let answer = Answer::new();
        self.tx.send(StoreRequest::Last(answer.clone())).unwrap();
        answer.await_answer()
    }
}

fn start_storage_loop<K: StringStoreKey, V: StringStoreValue>(
    rx: Receiver<StoreRequest<K, V>>,
    path: PathBuf,
) {
    std::thread::spawn(move || {
        let path = &path;
        std::fs::create_dir_all(path).unwrap();
        loop {
            match rx.recv() {
                Ok(StoreRequest::Get { key, answer }) => {
                    match std::fs::read_to_string(path.join(key.serialize_key())) {
                        Ok(data) => answer.answer(V::deserialize_value(&data)),
                        Err(_) => answer.answer(None),
                    }
                }
                Ok(StoreRequest::Insert { key, value, answer }) => {
                    if let Ok(mut file) = OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(path.join(key.serialize_key()))
                    {
                        file.write_all(value.serialize_value().as_bytes()).unwrap()
                    }

                    answer.answer(());
                }
                Ok(StoreRequest::Last(answer)) => match std::fs::read_dir(path) {
                    Ok(paths) => {
                        // Read the directory to file keys
                        let mut keys = paths
                            .filter_map(|path| match path {
                                Ok(dir_entry) => Some(K::deserialize_key(
                                    &dir_entry.file_name().into_string().unwrap(),
                                ))
                                .flatten(),
                                Err(_) => None,
                            })
                            .collect::<Vec<_>>();

                        // Sort
                        keys.sort();

                        // If the last one exists, try to read the value
                        match keys.last() {
                            Some(key) => {
                                match std::fs::read_to_string(path.join(key.serialize_key())) {
                                    Ok(data) => answer.answer(
                                        V::deserialize_value(&data)
                                            .map(|value| (key.clone(), value)),
                                    ),
                                    Err(_) => answer.answer(None),
                                }
                            }
                            None => answer.answer(None),
                        };
                    }
                    Err(_) => answer.answer(None),
                },
                Ok(StoreRequest::Len(answer)) => match std::fs::read_dir(path) {
                    Ok(paths) => answer.answer(paths.count()),
                    Err(_) => answer.answer(0),
                },
                Err(_) => break,
            }
        }
    });
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

// TODO: Unittests of FilesystemStore using tempdir
