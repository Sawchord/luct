//! This module contains the low-level call API
//!
//! Each function does exactly one call, parses and validates the
//! returned data.

use crate::{Client, ClientError, CtClient};
use base64::{Engine, prelude::BASE64_STANDARD};
use luct_core::{
    CertificateChain, CertificateError,
    store::Hashable,
    tree::{AuditProof, ConsistencyProof},
    v1::{
        SignedCertificateTimestamp, SignedTreeHead, TreeHead,
        responses::{GetProofByHashResponse, GetSthConsistencyResponse, GetSthResponse},
    },
};
use std::cmp::Ordering;

// TODO: Check that the error code is 200
// TODO: Introduce logging / tracing

impl<C: Client> CtClient<C> {
    pub async fn get_sth_v1(&self) -> Result<SignedTreeHead, ClientError> {
        self.assert_v1()?;
        let url = self.get_full_v1_url();

        let response = self.client.get(&url.join("get-sth").unwrap(), &[]).await?;
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
        self.assert_v1()?;

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
            .get(
                &url.join("get-sth-consistency").unwrap(),
                &[("first", &first_idx), ("second", &second_idx)],
            )
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

    pub async fn check_embedded_sct_inclusion_v1(
        &self,
        sct: &SignedCertificateTimestamp,
        sth: &SignedTreeHead,
        certificate_chain: &CertificateChain,
    ) -> Result<(), ClientError> {
        self.assert_v1()?;

        let leaf = certificate_chain
            .as_leaf_v1(sct, true)
            .map_err(CertificateError::from)?;
        let leaf_hash = leaf.hash();
        let leaf_hash: String = BASE64_STANDARD.encode(leaf_hash);

        let tree_size = sth.tree_size().to_string();

        let url = self.get_full_v1_url();
        let response = self
            .client
            .get(
                &url.join("get-proof-by-hash").unwrap(),
                &[("hash", &leaf_hash), ("tree_size", &tree_size)],
            )
            .await?;
        let response: GetProofByHashResponse =
            serde_json::from_str(&response).map_err(|_| ClientError::AuditProofError)?;
        let proof = AuditProof::try_from(response).map_err(|_| ClientError::AuditProofError)?;
        let tree_head = TreeHead::try_from(sth).map_err(|_| ClientError::AuditProofError)?;

        if !proof.validate(&tree_head, &leaf) {
            return Err(ClientError::AuditProofError);
        }

        Ok(())
    }
}

// TODO: Low level get entries call
// TODO: Low level get roots call

#[cfg(all(test, feature = "reqwest"))]
mod tests {
    use super::*;
    use crate::{CtClientConfig, reqwest::ReqwestClient};
    use luct_core::CtLogConfig;

    const ARGON2025H2: &str = "
        version = 1
        url = \"https://ct.googleapis.com/logs/us1/argon2025h2/\"
        key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEr+TzlCzfpie1/rJhgxnIITojqKk9VK+8MZoc08HjtsLzD8e5yjsdeWVhIiWCVk6Y6KomKTYeKGBv6xVu93zQug==\"
        mdd = 86400
    ";

    const ARGON2025H2_STH_0506: &str = "{
        \"tree_size\":1329315675,
        \"timestamp\":1751738269891,
        \"sha256_root_hash\":\"NEFqldTJt2+wE/aaaQuXeADdWVV8IGbwhLublI7QaMY=\",
        \"tree_head_signature\":\"BAMARjBEAiA9rna9/avaKTald7hHrldq8FfB4FDAaNyB44pplv71agIgeD0jj2AhLnvlaWavfFZ3BdUglauz36rFpGLYuLBs/O8=\"
    }";
    const CERT_CHAIN_GOOGLE_COM: &str = include_str!("../../testdata/google-chain.pem");

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn sth_consistency() {
        let client = get_client();

        let old_sth: GetSthResponse = serde_json::from_str(ARGON2025H2_STH_0506).unwrap();
        let old_sth = SignedTreeHead::from(old_sth);

        let new_sth = client.get_sth_v1().await.unwrap();
        client
            .check_consistency_v1(&old_sth, &new_sth)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn sct_inclusion() {
        let client = get_client();

        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let sth = client.get_sth_v1().await.unwrap();
        client
            .check_embedded_sct_inclusion_v1(&scts[0], &sth, &cert)
            .await
            .unwrap();
    }

    fn get_client() -> CtClient<ReqwestClient> {
        let config: CtLogConfig = toml::from_str(ARGON2025H2).unwrap();
        let client = ReqwestClient::new();
        CtClient::new(
            CtClientConfig {
                log: config,
                fetch_url: None,
            },
            client,
        )
    }
}
