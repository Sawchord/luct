use crate::{ScannerError, ScannerImpl, Validated, log::ScannerLog};
use luct_core::{
    Certificate,
    store::{OrderedStoreRead, StoreWrite},
    v1::SignedTreeHead,
};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

impl<S: ScannerImpl> ScannerLog<S> {
    /// Get a fresh STH
    ///
    /// Checks whether the latest STH is still new enough.
    /// If it is too old, it will fetch a fresh one
    ///
    /// If a cert is provided, then it will also check that the fresh STH is
    /// younger than the not_before value of the cert
    pub(crate) async fn get_fresh_sth(
        &self,
        now: SystemTime,
        cert: Option<&Certificate>,
    ) -> Result<Validated<SignedTreeHead>, ScannerError> {
        match self.try_get_fresh_sth(now, cert).await {
            Some(sth) => Ok(sth),
            None => self.update_sth().await,
        }
    }

    /// Updates the log to the newest STH
    ///
    /// Checks consistency to the last STH, if one exists
    pub(crate) async fn update_sth(&self) -> Result<Validated<SignedTreeHead>, ScannerError> {
        // NOTE: We hold the lock over the STH store while we fetch the new STH
        // This way, every request to the STH store will be queued until the update has finished
        // Most updates will want to have the updated store anyway, so this potentially reduces the
        // number of requests necessary
        let store = self.log.sth_store.lock().await;
        let new_sth = self.fetch_sth().await?;

        if let Some((_, old_sth)) = store.last().await
            && old_sth.tree_size() < new_sth.tree_size()
        {
            tracing::debug!(
                "Updating STH: Checking STH {} against old STH {}",
                new_sth.tree_size(),
                old_sth.tree_size()
            );

            match &self.log.tiles {
                Some(tiles) => tiles.check_sth_consistency(&old_sth, &new_sth).await?,
                None => {
                    self.log
                        .sth_client
                        .check_consistency_v1(&old_sth, &new_sth)
                        .await?
                }
            };
        };

        store.insert(new_sth.tree_size(), new_sth.clone()).await;

        Ok(new_sth)
    }

    async fn try_get_fresh_sth(
        &self,
        now: SystemTime,
        cert: Option<&Certificate>,
    ) -> Option<Validated<SignedTreeHead>> {
        let log_name = self.sth_client().log().description();

        // If we have no STH whatsoever, simply fetch it
        let Some(last_sth) = self.get_latest_sth().await else {
            tracing::debug!("No prior known STHs for {}", log_name);
            return None;
        };

        // Check if the update threshold has expired
        let sth_timestamp = UNIX_EPOCH + Duration::from_millis(last_sth.timestamp());
        if sth_timestamp + self.config.sth_update_threshold < now {
            tracing::debug!(
                "STH for {} needs update because update threshold has been met",
                log_name
            );
            return None;
        }

        if let Some(cert) = cert {
            // Update STH if cert is younger than latest STH
            let cert_timestamp = cert.get_validity_systemtime().0;
            if cert_timestamp > sth_timestamp {
                tracing::debug!(
                    "STH for {} needs update because certificate is newer than STH",
                    log_name
                );
                return None;
            }
        }

        Some(last_sth)
    }
}
