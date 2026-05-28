use crate::{Scanner, ScannerError, ScannerImpl, Validated, log::ScannerLog};
use luct_core::{Certificate, v1::SignedTreeHead};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

impl<S: ScannerImpl> Scanner<S> {
    /// Get a fresh STH
    ///
    /// Checks whether the latest STH is still new enough.
    /// If it is too old, it will fetch a fresh one
    pub(crate) async fn update_fresh_sth(
        &self,
        now: SystemTime,
        log: &ScannerLog<S>,
        cert: &Certificate,
    ) -> Result<Validated<SignedTreeHead>, ScannerError> {
        match self.get_fresh_sth(now, log, cert) {
            Some(sth) => Ok(sth),
            None => log.update_sth().await,
        }
    }

    pub(crate) fn get_fresh_sth(
        &self,
        now: SystemTime,
        log: &ScannerLog<S>,
        cert: &Certificate,
    ) -> Option<Validated<SignedTreeHead>> {
        let log_name = log.client().log().description();

        // If we have no STH whatsoever, simply fetch it
        let Some(last_sth) = log.get_latest_sth() else {
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

        // Update STH if cert is younger than latest STH
        let cert_timestamp = cert.get_validity().0;
        let cert_timestamp =
            UNIX_EPOCH + Duration::from_millis(cert_timestamp.timestamp_millis() as u64);
        if cert_timestamp > sth_timestamp {
            tracing::debug!(
                "STH for {} needs update because certificate is newer than STH",
                log_name
            );
            return None;
        }

        Some(last_sth)
    }
}
