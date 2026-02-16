//! This module contains the low-level call API
//!
//! Each function does exactly one call, parses and validates the
//! returned data.

use crate::{Client, ClientError, CtClient};
use base64::{Engine, prelude::BASE64_STANDARD};
use luct_core::{
    Certificate, Version,
    store::Hashable,
    tree::{AuditProof, ConsistencyProof, TreeHead},
    v1::{
        MerkleTreeLeaf, SignedCertificateTimestamp, SignedTreeHead,
        responses::{
            GetProofByHashResponse, GetRootsResponse, GetSthConsistencyResponse, GetSthResponse,
        },
    },
};
use std::cmp::Ordering;
use url::Url;

impl<C: Client> CtClient<C> {
    #[tracing::instrument(level = "trace")]
    pub async fn get_sth_v1(&self) -> Result<SignedTreeHead, ClientError> {
        self.assert_v1()?;
        let url = self.get_full_v1_url().join("get-sth").unwrap();

        // Fetch and parse the signed tree head
        let (status, response) = self.client.get(&url, &[]).await?;
        self.check_status(&url, status, &response)?;
        let response: GetSthResponse = serde_json::from_str(&response)?;
        let response = SignedTreeHead::try_from(response).map_err(|_| ClientError::SthError)?;

        // Validate tree head signature against key
        self.log
            .validate_sth_v1(&response)
            .map_err(|err| ClientError::SignatureValidationFailed("STH", err))?;

        tracing::debug!("fetched and validated STH {:?} from url {}", response, url);

        Ok(response)
    }

    #[tracing::instrument(level = "trace")]
    pub async fn update_sth_v1(
        &self,
        old_sth: Option<&SignedTreeHead>,
    ) -> Result<SignedTreeHead, ClientError> {
        let new_sth = self.get_sth_v1().await?;

        // If we have no old sth, simply return the new one
        let Some(old_sth) = old_sth else {
            return Ok(new_sth);
        };

        if old_sth == &new_sth {
            return Ok(new_sth);
        }

        self.check_consistency_v1(old_sth, &new_sth).await?;

        Ok(new_sth)
    }

    #[tracing::instrument(level = "trace")]
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
        let url = self.get_full_v1_url().join("get-sth-consistency").unwrap();
        let (status, response) = self
            .client
            .get(&url, &[("first", &first_idx), ("second", &second_idx)])
            .await?;
        self.check_status(&url, status, &response)?;

        let response: GetSthConsistencyResponse = serde_json::from_str(&response)?;
        let proof =
            ConsistencyProof::try_from(response).map_err(ClientError::ConsistencyProofError)?;

        let first = TreeHead::from(first);
        let second = TreeHead::from(second);

        // Validate inclusion proof
        proof
            .validate(&first, &second)
            .map_err(ClientError::ConsistencyProofError)?;

        tracing::debug!(
            "fetched and validated consistency proof for tree sizes {} to {}",
            first.tree_size(),
            second.tree_size()
        );

        Ok(())
    }

    #[tracing::instrument(level = "trace")]
    pub async fn check_sct_inclusion_v1(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &SignedTreeHead,
        leaf: &MerkleTreeLeaf,
    ) -> Result<(), ClientError> {
        self.assert_v1()?;

        let leaf_hash = leaf.hash();
        let leaf_hash: String = BASE64_STANDARD.encode(leaf_hash);

        let tree_size = sth.tree_size().to_string();

        // Fetch and parse inclusion proof
        let url = self.get_full_v1_url().join("get-proof-by-hash").unwrap();
        let (status, response) = self
            .client
            .get(&url, &[("hash", &leaf_hash), ("tree_size", &tree_size)])
            .await?;
        self.check_status(&url, status, &response)?;

        let response: GetProofByHashResponse = serde_json::from_str(&response)?;
        let proof = AuditProof::try_from(response).map_err(ClientError::AuditProofError)?;
        let tree_head = TreeHead::from(sth);

        // Validate inclusion proof
        proof
            .validate(&tree_head, leaf)
            .map_err(ClientError::AuditProofError)?;

        tracing::debug!(
            "fetched and validated embedded SCT {:?} for tree size {}",
            sct,
            sth.tree_size()
        );

        Ok(())
    }

    #[tracing::instrument(level = "trace")]
    pub async fn get_roots_v1(&self) -> Result<Vec<Certificate>, ClientError> {
        self.assert_v1()?;

        let url = self.get_full_v1_url().join("get-roots").unwrap();
        let (status, response) = self.client.get(&url, &[]).await?;
        self.check_status(&url, status, &response)?;

        let response: GetRootsResponse = serde_json::from_str(&response)?;

        tracing::debug!("fetched roots from url {}", url);

        Ok((&response).into())
    }

    fn get_full_v1_url(&self) -> Url {
        let base_url = self.log().config().fetch_url();
        base_url.join("ct/v1/").unwrap()
    }

    pub(crate) fn assert_v1(&self) -> Result<(), ClientError> {
        match self.log().config().version() {
            Version::V1 => Ok(()),
            #[allow(unreachable_patterns)]
            _ => Err(ClientError::UnsupportedVersion),
        }
    }
}

// TODO: Low level get entries call

#[cfg(all(test, feature = "reqwest"))]
mod tests {
    use super::*;
    use crate::reqwest::ReqwestClient;
    use luct_core::{
        CertificateChain, CtLogConfig,
        v1::{SignedTreeHead, responses::GetSthResponse},
    };

    const ARGON2025H2: &str = "{
        \"description\": \"Google Argon\",
        \"version\": 1,
        \"url\": \"https://ct.googleapis.com/logs/us1/argon2025h2/\",
        \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEr+TzlCzfpie1/rJhgxnIITojqKk9VK+8MZoc08HjtsLzD8e5yjsdeWVhIiWCVk6Y6KomKTYeKGBv6xVu93zQug==\",
        \"mmd\": 86400
        }
    ";

    const ARGON2025H2_STH_0506: &str = "{
        \"tree_size\":1329315675,
        \"timestamp\":1751738269891,
        \"sha256_root_hash\":\"NEFqldTJt2+wE/aaaQuXeADdWVV8IGbwhLublI7QaMY=\",
        \"tree_head_signature\":\"BAMARjBEAiA9rna9/avaKTald7hHrldq8FfB4FDAaNyB44pplv71agIgeD0jj2AhLnvlaWavfFZ3BdUglauz36rFpGLYuLBs/O8=\"
    }";
    const CERT_CHAIN_GOOGLE_COM: &str = include_str!("../../../testdata/google-chain.pem");

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn sth_consistency() {
        let client = get_client();

        let old_sth: GetSthResponse = serde_json::from_str(ARGON2025H2_STH_0506).unwrap();
        let old_sth = SignedTreeHead::try_from(old_sth).unwrap();

        client.update_sth_v1(Some(&old_sth)).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn sct_inclusion() {
        let client = get_client();

        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        cert.verify_chain().unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let sth = client.get_sth_v1().await.unwrap();
        client
            .check_sct_inclusion_v1(&scts[0], &sth, &cert.as_leaf_v1(&scts[0], true).unwrap())
            .await
            .unwrap();
    }

    fn get_client() -> CtClient<ReqwestClient> {
        let config: CtLogConfig = serde_json::from_str(ARGON2025H2).unwrap();
        let client = ReqwestClient::new();
        CtClient::new(config, client)
    }

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn get_roots() {
        let client = get_client();

        let roots = client.get_roots_v1().await.unwrap();
        assert!(!roots.is_empty())
    }
}
