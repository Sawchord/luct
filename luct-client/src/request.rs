//! This module contains the low-level call API
//!
//! Each function does exactly one call, parses and validates the
//! returned data.

use crate::{Client, ClientError, CtClient};
use base64::{Engine, prelude::BASE64_STANDARD};
use luct_core::{
    store::Hashable, tree::{AuditProof, ConsistencyProof, TreeHead}, v1::{
        responses::{GetProofByHashResponse, GetSthConsistencyResponse, GetSthResponse}, SignedCertificateTimestamp, SignedTreeHead
    }, CertificateChain, CertificateError
};
use std::cmp::Ordering;

// TODO: Introduce logging / tracing

impl<C: Client> CtClient<C> {
    pub async fn get_sth_v1(&self) -> Result<SignedTreeHead, ClientError> {
        self.assert_v1()?;
        let url = self.get_full_v1_url();

        // Fetch and parse the signed tree head
        let (status, response) = self.client.get(&url.join("get-sth").unwrap(), &[]).await?;
        self.check_status(status, &response)?;
        let response: GetSthResponse = serde_json::from_str(&response)?;
        let response = SignedTreeHead::from(response);

        // Validate tree head signature against key
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
        self.assert_v1()?;

        // Swap first and second if second < first
        let (first, second) = match first.tree_size().cmp(&second.tree_size()) {
            Ordering::Less => (first, second),
            Ordering::Equal => return Ok(()),
            Ordering::Greater => (second, first),
        };

        let first_idx = first.tree_size().to_string();
        let second_idx = second.tree_size().to_string();

        // Fetch and parse inclusion proof
        let url = self.get_full_v1_url();
        let (status, response) = self
            .client
            .get(
                &url.join("get-sth-consistency").unwrap(),
                &[("first", &first_idx), ("second", &second_idx)],
            )
            .await?;
        self.check_status(status, &response)?;

        let response: GetSthConsistencyResponse = serde_json::from_str(&response)?;
        let proof =
            ConsistencyProof::try_from(response).map_err(|_| ClientError::ConsistencyProofError)?;

        let first = TreeHead::try_from(first).map_err(|_| ClientError::ConsistencyProofError)?;
        let second = TreeHead::try_from(second).map_err(|_| ClientError::ConsistencyProofError)?;

        // Validate inclusion proof
        if !proof.validate(&first, &second) {
            return Err(ClientError::ConsistencyProofError);
        }

        Ok(())
    }

    pub async fn check_embedded_sct_inclusion_v1(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &SignedTreeHead,
        certificate_chain: &CertificateChain,
    ) -> Result<(), ClientError> {
        self.assert_v1()?;

        // Compute tree leaf hash
        let leaf = certificate_chain
            .as_leaf_v1(sct, true)
            .map_err(CertificateError::from)?;
        let leaf_hash = leaf.hash();
        let leaf_hash: String = BASE64_STANDARD.encode(leaf_hash);

        let tree_size = sth.tree_size().to_string();

        // Fetch and parse inclusion proof
        let url = self.get_full_v1_url();
        let (status, response) = self
            .client
            .get(
                &url.join("get-proof-by-hash").unwrap(),
                &[("hash", &leaf_hash), ("tree_size", &tree_size)],
            )
            .await?;
        self.check_status(status, &response)?;

        let response: GetProofByHashResponse =
            serde_json::from_str(&response).map_err(|_| ClientError::AuditProofError)?;
        let proof = AuditProof::try_from(response).map_err(|_| ClientError::AuditProofError)?;
        let tree_head = TreeHead::try_from(sth).map_err(|_| ClientError::AuditProofError)?;

        // Validate inclusion proof
        if !proof.validate(&tree_head, &leaf) {
            return Err(ClientError::AuditProofError);
        }

        Ok(())
    }
}

// TODO: Low level get entries call
// TODO: Low level get roots call
