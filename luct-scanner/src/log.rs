use crate::{Conclusion, Scanner, lead::EmbeddedSct, log::tiling::TileFetcher};
use luct_client::{Client, ClientError, CtClient};
use luct_core::{
    Certificate, CertificateChain,
    store::{Hashable, OrderedStore, Store},
    v1::{SignedCertificateTimestamp, SignedTreeHead},
};
use std::sync::Arc;

pub(crate) mod builder;
pub(crate) mod tiling;

/// Internal structure holding references to per log
/// clients and stores
pub(crate) struct ScannerLog<C> {
    log: Arc<ScannerLogInner<C>>,
    tiles: Option<TileFetcher<C>>,
}

pub(crate) struct ScannerLogInner<C> {
    name: String,
    client: CtClient<C>,
    sth_store: Box<dyn OrderedStore<u64, SignedTreeHead>>,
    root_keys: Box<dyn Store<Vec<u8>, ()>>,
}

impl<C: Client> ScannerLog<C> {
    pub(crate) fn client(&self) -> &CtClient<C> {
        &self.log.client
    }

    pub(crate) async fn investigate_embedded_sct(
        &self,
        sct: &EmbeddedSct,
        scanner: &Scanner<C>,
    ) -> Result<Conclusion, ClientError> {
        let EmbeddedSct { sct, chain } = sct;

        if scanner.sct_cache.get(&sct.hash()).is_some() {
            return Ok(Conclusion::Safe(format!(
                "cache returned valid SCT of \"{}\"",
                self.log.name
            )));
        }

        self.check_embedded_sct_inclusion(sct, chain).await?;

        // Check that the roots certificate is included in the list of allowed roots
        let root_validation = self.validate_root(chain.root()).await?;
        if !root_validation.is_safe() {
            return Ok(root_validation);
        }

        scanner.sct_cache.insert(sct.hash(), sct.clone());

        Ok(Conclusion::Safe(format!(
            "\"{}\" returned a valid audit proof",
            self.log.name
        )))
    }

    async fn check_embedded_sct_inclusion(
        &self,
        sct: &SignedCertificateTimestamp,
        chain: &Arc<CertificateChain>,
    ) -> Result<(), ClientError> {
        if sct.timestamp() > self.latest_sth().await?.timestamp() {
            self.update_sth().await?;
        }
        let sth = self.latest_sth().await?;

        match &self.tiles {
            Some(tiles) => tiles.check_embdedded_sct_inclusion(sct, &sth, chain).await,
            None => {
                self.log
                    .client
                    .check_embedded_sct_inclusion_v1(sct, &sth, chain)
                    .await
            }
        }
    }

    /// Returns the latests STH, if it exists, fetches it otherwise
    async fn latest_sth(&self) -> Result<SignedTreeHead, ClientError> {
        match self.log.sth_store.last() {
            Some((_, sth)) => Ok(sth),
            None => {
                let sth = self.get_sth().await?;
                self.log.sth_store.insert(sth.tree_size(), sth.clone());
                Ok(sth)
            }
        }
    }

    /// Updates the log to the newest STH, checks consistency if possible
    pub(crate) async fn update_sth(&self) -> Result<(), ClientError> {
        let new_sth = self.get_sth().await?;

        if let Some((_, old_sth)) = self.log.sth_store.last() {
            self.log
                .client
                .check_consistency_v1(&old_sth, &new_sth)
                .await?;
        };
        self.log.sth_store.insert(new_sth.tree_size(), new_sth);

        Ok(())
    }

    async fn get_sth(&self) -> Result<SignedTreeHead, ClientError> {
        match &self.tiles {
            Some(_) => self.log.client.get_checkpoint().await,
            None => self.log.client.get_sth_v1().await,
        }
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

        if self.log.root_keys.get(&key_id).is_some() {
            return Ok(Conclusion::Safe(
                "Fingerprint matches allowed roots".to_string(),
            ));
        }

        self.update_roots().await?;

        match self.log.root_keys.get(&key_id) {
            Some(()) => Ok(Conclusion::Safe("Root matches allowed roots".to_string())),
            None => Ok(Conclusion::Unsafe(format!(
                "Root is not included in the list of allowed roots of log {}",
                self.log.name
            ))),
        }
    }

    async fn update_roots(&self) -> Result<(), ClientError> {
        let certs = self.log.client.get_roots_v1().await?;
        for cert in certs {
            if let Some(key_id) = cert.get_subject_key_info() {
                self.log.root_keys.insert(key_id, ());
            }
        }

        Ok(())
    }
}
