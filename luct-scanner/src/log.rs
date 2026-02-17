use crate::{Conclusion, Scanner, ScannerError, lead::EmbeddedSct, log::tiling::TileFetcher};
use luct_client::{Client, CtClient};
use luct_core::{
    Certificate, CertificateChain, CertificateError,
    store::{Hashable, OrderedStore, Store},
    v1::{SignedCertificateTimestamp, SignedTreeHead},
};
use std::{fmt, sync::Arc};

pub(crate) mod builder;
pub(crate) mod tiling;

/// Internal structure holding references to per log
/// clients and stores
#[derive(Debug)]
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

impl<C> fmt::Debug for ScannerLogInner<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScannerLogInner")
            .field("name", &self.name)
            .finish()
    }
}

impl<C: Client> ScannerLog<C> {
    pub(crate) fn client(&self) -> &CtClient<C> {
        &self.log.client
    }

    pub(crate) async fn investigate_embedded_sct(
        &self,
        sct: &EmbeddedSct,
        scanner: &Scanner<C>,
    ) -> Result<Conclusion, ScannerError> {
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

    #[tracing::instrument(level = "trace")]
    async fn check_embedded_sct_inclusion(
        &self,
        sct: &SignedCertificateTimestamp,
        chain: &Arc<CertificateChain>,
    ) -> Result<(), ScannerError> {
        if sct.timestamp() > self.latest_sth().await?.timestamp() {
            self.update_sth().await?;
        }
        let sth = self.latest_sth().await?;

        tracing::debug!(
            "Checking embedded SCT against log {} at tree size {}",
            self.log.name,
            sth.tree_size()
        );

        // Compute tree leaf hash
        let leaf = chain
            .as_leaf_v1(sct, true)
            .map_err(CertificateError::from)?;

        match &self.tiles {
            Some(tiles) => Ok(tiles
                .check_embdedded_sct_inclusion(sct, &sth, &leaf)
                .await?),
            None => Ok(self
                .log
                .client
                .check_sct_inclusion_v1(sct, &sth, &leaf)
                .await?),
        }
    }

    /// Returns the latests STH, if it exists, fetches it otherwise
    #[tracing::instrument(level = "trace")]
    async fn latest_sth(&self) -> Result<SignedTreeHead, ScannerError> {
        match self.log.sth_store.last() {
            Some((_, sth)) => Ok(sth),
            None => {
                let sth = self.get_sth().await?;
                self.log.sth_store.insert(sth.tree_size(), sth.clone());
                Ok(sth)
            }
        }
    }

    /// Updates the log to the newest STH
    ///
    /// Checks consistency to the last STH, of one exists
    #[tracing::instrument(level = "trace")]
    pub(crate) async fn update_sth(&self) -> Result<(), ScannerError> {
        let new_sth = self.get_sth().await?;

        if let Some((_, old_sth)) = self.log.sth_store.last() {
            tracing::debug!(
                "Updating STH: Checking STH {} against old STH {}",
                new_sth.tree_size(),
                old_sth.tree_size()
            );

            match &self.tiles {
                Some(tiles) => tiles.check_sth_consistency(&old_sth, &new_sth).await?,
                None => {
                    self.log
                        .client
                        .check_consistency_v1(&old_sth, &new_sth)
                        .await?
                }
            };
        };
        self.log.sth_store.insert(new_sth.tree_size(), new_sth);

        Ok(())
    }

    #[tracing::instrument(level = "trace")]
    async fn get_sth(&self) -> Result<SignedTreeHead, ScannerError> {
        tracing::debug!("Fetching new STH of log {}", self.log.name);
        match &self.tiles {
            Some(_) => Ok(self.log.client.get_checkpoint().await?),
            None => Ok(self.log.client.get_sth_v1().await?),
        }
    }

    #[tracing::instrument(level = "trace")]
    async fn validate_root(&self, root: &Certificate) -> Result<Conclusion, ScannerError> {
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

    #[tracing::instrument(level = "trace")]
    async fn update_roots(&self) -> Result<(), ScannerError> {
        let certs = self.log.client.get_roots_v1().await?;
        for cert in certs {
            if let Some(key_id) = cert.get_subject_key_info() {
                self.log.root_keys.insert(key_id, ());
            }
        }

        Ok(())
    }
}
