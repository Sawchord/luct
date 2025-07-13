use crate::{
    tree::{AuditProof, ConsistencyProof, TreeHead},
    v1::{
        responses::{GetProofByHashResponse, GetSthConsistencyResponse},
        sth::{SignedTreeHead, TreeHeadSignature},
    },
};

impl TreeHead {
    /// Validate the consistency of a [`GetSthResponse`] using a [`GetSthConsistencyResponse`]
    ///
    /// If this call returns true, `new_sth` is valid and can be used as the new [`TreeHead`]
    pub fn validate_consistency(
        &self,
        new_sth: &SignedTreeHead,
        proof: GetSthConsistencyResponse,
    ) -> bool {
        let Ok(new_tree_head) = TreeHead::try_from(new_sth) else {
            return false;
        };

        let Ok(proof) = ConsistencyProof::try_from(proof) else {
            return false;
        };

        proof.validate(self, &new_tree_head)
    }
}

impl TryFrom<GetSthConsistencyResponse> for ConsistencyProof {
    type Error = ();

    fn try_from(value: GetSthConsistencyResponse) -> Result<Self, Self::Error> {
        Ok(ConsistencyProof {
            path: value
                .consistency
                .into_iter()
                .map(|elem| elem.0.try_into().map_err(|_| ()))
                .collect::<Result<Vec<[u8; 32]>, ()>>()?,
        })
    }
}

impl From<TreeHeadSignature> for TreeHead {
    fn from(value: TreeHeadSignature) -> Self {
        Self {
            tree_size: value.tree_size,
            head: value.sha256_root_hash,
        }
    }
}

impl TryFrom<&SignedTreeHead> for TreeHead {
    type Error = ();

    fn try_from(value: &SignedTreeHead) -> Result<Self, Self::Error> {
        let sth = TreeHeadSignature::try_from(value)?;
        Ok(sth.into())
    }
}

impl TryFrom<GetProofByHashResponse> for AuditProof {
    type Error = ();

    fn try_from(value: GetProofByHashResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            index: value.leaf_index,
            path: value
                .audit_path
                .into_iter()
                .map(|elem| elem.0.try_into().map_err(|_| ()))
                .collect::<Result<Vec<[u8; 32]>, ()>>()?,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        CertificateChain,
        tests::{
            ARGON2025H1_STH2806, ARGON2025H1_STH2906, CERT_CHAIN_GOOGLE_COM, GOOGLE_AUDIT_PROOF,
            GOOGLE_STH_CONSISTENCY_PROOF, get_log_argon2025h2,
        },
        v1::responses::{GetProofByHashResponse, GetSthResponse},
    };

    const ARGON2025H2_STH_0506: &str = "{
        \"tree_size\":1329315675,
        \"timestamp\":1751738269891,
        \"sha256_root_hash\":\"NEFqldTJt2+wE/aaaQuXeADdWVV8IGbwhLublI7QaMY=\",
        \"tree_head_signature\":\"BAMARjBEAiA9rna9/avaKTald7hHrldq8FfB4FDAaNyB44pplv71agIgeD0jj2AhLnvlaWavfFZ3BdUglauz36rFpGLYuLBs/O8=\"
    }";

    #[test]
    fn validate_sth_consistency() {
        let old_sth: GetSthResponse = serde_json::from_str(ARGON2025H1_STH2806).unwrap();
        let old_tree_head = TreeHead::try_from(&old_sth.into()).unwrap();

        let new_sth: GetSthResponse = serde_json::from_str(ARGON2025H1_STH2906).unwrap();
        let proof: GetSthConsistencyResponse =
            serde_json::from_str(GOOGLE_STH_CONSISTENCY_PROOF).unwrap();

        assert!(old_tree_head.validate_consistency(&new_sth.into(), proof))
    }

    #[test]
    fn audit_sct() {
        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let log = get_log_argon2025h2();
        assert_eq!(log.log_id_v1(), scts[0].log_id());

        let leaf = cert.as_leaf_v1(&scts[0], true).unwrap();

        let sth: GetSthResponse = serde_json::from_str(ARGON2025H2_STH_0506).unwrap();
        let tree_head = TreeHead::try_from(&sth.into()).unwrap();

        let audit_proof: GetProofByHashResponse = serde_json::from_str(GOOGLE_AUDIT_PROOF).unwrap();

        assert!(leaf.validate_inclusion(&tree_head, audit_proof))
    }
}
