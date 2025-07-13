use std::cmp::Ordering;

use crate::{Client, ClientError, CtClient};
use luct_core::{
    tree::ConsistencyProof,
    v1::{
        SignedTreeHead, TreeHead,
        responses::{GetSthConsistencyResponse, GetSthResponse},
    },
};

impl<C: Client> CtClient<C> {
    pub async fn get_sth_v1(&self) -> Result<SignedTreeHead, ClientError> {
        self.assert_v1()?;
        let url = self.get_full_v1_url();

        let response = self.client.get(&url, &[]).await?;
        let response: GetSthResponse = serde_json::from_str(&response)?;
        let response = SignedTreeHead::from(response);

        self.log
            .validate_sth_v1(&response)
            .map_err(|err| ClientError::SignatureValidationFailed("STH", err))?;

        Ok(response)
    }

    pub async fn check_consistency_v1(
        &self,
        first: &SignedTreeHead,
        second: &SignedTreeHead,
    ) -> Result<(), ClientError> {
        let (first, second) = match first.tree_size().cmp(&second.tree_size()) {
            Ordering::Less => (first, second),
            Ordering::Equal => return Ok(()),
            Ordering::Greater => (second, first),
        };

        let first_idx = first.tree_size().to_string();
        let second_idx = second.tree_size().to_string();

        let url = self.get_full_v1_url();
        let response = self
            .client
            .get(&url, &[("first", &first_idx), ("second", &second_idx)])
            .await?;

        let response: GetSthConsistencyResponse = serde_json::from_str(&response)?;
        let proof =
            ConsistencyProof::try_from(response).map_err(|_| ClientError::ConsistencyProofError)?;

        let first = TreeHead::try_from(first).map_err(|_| ClientError::ConsistencyProofError)?;
        let second = TreeHead::try_from(second).map_err(|_| ClientError::ConsistencyProofError)?;

        if !proof.validate(&first, &second) {
            return Err(ClientError::ConsistencyProofError);
        }

        Ok(())
    }
}
