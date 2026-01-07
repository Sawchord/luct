use crate::{Conclusion, Scanner, lead::EmbeddedSct};
use luct_client::{Client, ClientError, CtClient};
use luct_core::{
    Certificate, CtLog, CtLogConfig,
    store::{Hashable, OrderedStore, Store},
    v1::SignedTreeHead,
};

pub struct Log {
    pub(crate) name: String,
    pub(crate) config: CtLogConfig,
    pub(crate) sth_store: Option<Box<dyn OrderedStore<u64, SignedTreeHead>>>,
    pub(crate) root_keys: Option<Box<dyn Store<Vec<u8>, ()>>>,
}

impl Log {
    pub fn new(log: &CtLog) -> Self {
        Self {
            name: log.description().to_string(),
            config: log.config().clone(),
            sth_store: None,
            root_keys: None,
        }
    }

    pub fn with_sth_store(
        mut self,
        store: impl OrderedStore<u64, SignedTreeHead> + 'static,
    ) -> Self {
        self.sth_store = Some(Box::new(store) as _);
        self
    }

    pub fn with_root_key_store(mut self, store: impl Store<Vec<u8>, ()> + 'static) -> Self {
        self.root_keys = Some(Box::new(store) as _);
        self
    }
}

/// Internal structure holding references to per log
/// clients and stores
pub(crate) struct ScannerLog<C> {
    pub(crate) name: String,
    pub(crate) client: CtClient<C>,
    pub(crate) sth_store: Box<dyn OrderedStore<u64, SignedTreeHead>>,
    pub(crate) root_keys: Box<dyn Store<Vec<u8>, ()>>,
}

impl<C: Client> ScannerLog<C> {
    pub(crate) async fn investigate_embedded_sct(
        &self,
        sct: &EmbeddedSct,
        scanner: &Scanner<C>,
    ) -> Result<Conclusion, ClientError> {
        let EmbeddedSct { sct, chain } = sct;

        if scanner.sct_cache.get(&sct.hash()).is_some() {
            return Ok(Conclusion::Safe(format!(
                "cache returned valid SCT of \"{}\"",
                self.name
            )));
        }

        if sct.timestamp() > self.latest_sth().await?.timestamp() {
            self.update_sth().await?;
        }
        let sth = self.latest_sth().await?;

        self.client
            .check_embedded_sct_inclusion_v1(sct, &sth, chain)
            .await?;

        // Check that the roots certificate is included in the list of allowed roots
        let root_validation = self.validate_root(chain.root()).await?;
        if !root_validation.is_safe() {
            return Ok(root_validation);
        }

        scanner.sct_cache.insert(sct.hash(), sct.clone());

        Ok(Conclusion::Safe(format!(
            "\"{}\" returned a valid audit proof",
            self.name
        )))
    }

    /// Returns the latests STH, if it exists, fetches it otherwise
    async fn latest_sth(&self) -> Result<SignedTreeHead, ClientError> {
        match self.sth_store.last() {
            Some((_, sth)) => Ok(sth),
            None => {
                let sth = self.client.get_sth_v1().await?;
                self.sth_store.insert(sth.tree_size(), sth.clone());
                Ok(sth)
            }
        }
    }

    /// Updates the log to the newest STH, checks consistency if possible
    pub(crate) async fn update_sth(&self) -> Result<(), ClientError> {
        let new_sth = self.client.get_sth_v1().await?;

        if let Some((_, old_sth)) = self.sth_store.last() {
            self.client.check_consistency_v1(&old_sth, &new_sth).await?;
        };
        self.sth_store.insert(new_sth.tree_size(), new_sth);

        Ok(())
    }

    async fn validate_root(&self, root: &Certificate) -> Result<Conclusion, ClientError> {
        let Some(key_id) = root
            .get_authority_key_info()
            .or_else(|| root.get_subject_key_info())
        else {
            return Ok(Conclusion::Inconclusive(
                "Certificate chain is not RFC5280 compliant".to_string(),
            ));
        };

        if self.root_keys.get(&key_id).is_some() {
            return Ok(Conclusion::Safe(
                "Fingerprint matches allowed roots".to_string(),
            ));
        }

        self.update_roots().await?;

        match self.root_keys.get(&key_id) {
            Some(()) => Ok(Conclusion::Safe("Root matches allowed roots".to_string())),
            None => Ok(Conclusion::Unsafe(format!(
                "Root is not included in the list of allowed roots of log {}",
                self.name
            ))),
        }
    }

    async fn update_roots(&self) -> Result<(), ClientError> {
        let certs = self.client.get_roots_v1().await?;
        for cert in certs {
            if let Some(key_id) = cert.get_subject_key_info() {
                self.root_keys.insert(key_id, ());
            }
        }

        Ok(())
    }
}
