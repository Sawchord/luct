use crate::{ScannerError, ScannerImpl, Validated, log::ScannerLog};
use luct_core::{
    store::{OrderedStoreRead, StoreWrite},
    v1::SignedTreeHead,
};

impl<S: ScannerImpl> ScannerLog<S> {
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
}
