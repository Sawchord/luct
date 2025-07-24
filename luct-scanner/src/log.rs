use luct_client::{Client, ClientError, CtClient};
use luct_core::{store::OrderedStore, v1::SignedTreeHead};

use crate::{Conclusion, ScannerError, lead::EmbeddedSct};

pub(crate) struct ScannerLog<C> {
    pub(crate) name: String,
    pub(crate) client: CtClient<C>,
    pub(crate) sth_store: Box<dyn OrderedStore<u64, SignedTreeHead>>,
    // TODO: Supported root fingerprints
}

impl<C: Client> ScannerLog<C> {
    /// Returns the latests STH, if it exists
    pub(crate) async fn latest_sth(&self) -> Result<SignedTreeHead, ScannerError> {
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
    pub(crate) async fn update_sth(&self) -> Result<(), ScannerError> {
        let new_sth = self.client.get_sth_v1().await?;

        if let Some((_, old_sth)) = self.sth_store.last() {
            self.client.check_consistency_v1(&old_sth, &new_sth).await?;
        };
        self.sth_store.insert(new_sth.tree_size(), new_sth);

        Ok(())
    }

    pub(crate) async fn investigate_embedded_sct(
        &self,
        sct: &EmbeddedSct,
    ) -> Result<Conclusion, ScannerError> {
        let EmbeddedSct { sct, chain } = sct;

        if sct.timestamp() > self.latest_sth().await?.timestamp() {
            self.update_sth().await?;
        }
        let sth = self.latest_sth().await?;

        match self
            .client
            .check_embedded_sct_inclusion_v1(sct, &sth, chain)
            .await
        {
            Ok(()) => Ok(Conclusion::Safe(format!(
                "\"{}\" returned a valid audit proof",
                self.name
            ))),
            Err(ClientError::AuditProofError) => Ok(Conclusion::Unsafe(format!(
                "\"{}\" returned an audit proof that failed verification",
                self.name
            ))),
            Err(err) => Err(ScannerError::from(err)),
        }
    }
}
