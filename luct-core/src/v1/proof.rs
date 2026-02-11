use crate::{
    tree::{AuditProof, ConsistencyProof, ProofValidationError, TreeHead},
    v1::{
        responses::{GetProofByHashResponse, GetSthConsistencyResponse},
        sth::{SignedTreeHead, TreeHeadSignature},
    },
};

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

impl From<&SignedTreeHead> for TreeHead {
    fn from(value: &SignedTreeHead) -> Self {
        let sth = TreeHeadSignature::from(value);
        sth.into()
    }
}

impl TryFrom<GetProofByHashResponse> for AuditProof {
    type Error = ProofValidationError;

    fn try_from(value: GetProofByHashResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            index: value.leaf_index,
            path: value
                .audit_path
                .into_iter()
                .map(|elem| {
                    elem.0.try_into().map_err(|vec: Vec<u8>| {
                        ProofValidationError::InvalidHashLength {
                            expected: 32,
                            received: vec.len(),
                        }
                    })
                })
                .collect::<Result<Vec<[u8; 32]>, ProofValidationError>>()?,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        CertificateChain,
        tests::{
            ARGON2025H1_STH2806, ARGON2025H1_STH2906, CERT_CHAIN_GOOGLE_COM, get_log_argon2025h2,
        },
        v1::responses::{GetProofByHashResponse, GetSthResponse},
    };

    const GOOGLE_AUDIT_PROOF: &str =
        include_str!("../../../testdata/google-precert-audit-proof.json");
    const GOOGLE_STH_CONSISTENCY_PROOF: &str =
        include_str!("../../../testdata/sth-consistency-proof.json");

    const ARGON2025H2_STH_0506: &str = "{
        \"tree_size\":1329315675,
        \"timestamp\":1751738269891,
        \"sha256_root_hash\":\"NEFqldTJt2+wE/aaaQuXeADdWVV8IGbwhLublI7QaMY=\",
        \"tree_head_signature\":\"BAMARjBEAiA9rna9/avaKTald7hHrldq8FfB4FDAaNyB44pplv71agIgeD0jj2AhLnvlaWavfFZ3BdUglauz36rFpGLYuLBs/O8=\"
    }";

    #[test]
    fn validate_sth_consistency() {
        let old_sth: GetSthResponse = serde_json::from_str(ARGON2025H1_STH2806).unwrap();
        let old_tree_head = TreeHead::from(&old_sth.try_into().unwrap());

        let new_sth: GetSthResponse = serde_json::from_str(ARGON2025H1_STH2906).unwrap();
        let proof: GetSthConsistencyResponse =
            serde_json::from_str(GOOGLE_STH_CONSISTENCY_PROOF).unwrap();
        let proof = ConsistencyProof::try_from(proof).unwrap();

        assert!(proof.validate(
            &old_tree_head,
            &TreeHead::from(&new_sth.try_into().unwrap())
        ))
    }

    #[test]
    fn audit_sct() {
        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        cert.verify_chain().unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let log = get_log_argon2025h2();
        assert_eq!(log.log_id(), &scts[0].log_id());

        let leaf = cert.as_leaf_v1(&scts[0], true).unwrap();

        let sth: GetSthResponse = serde_json::from_str(ARGON2025H2_STH_0506).unwrap();
        let tree_head = TreeHead::from(&sth.try_into().unwrap());

        let audit_proof: GetProofByHashResponse = serde_json::from_str(GOOGLE_AUDIT_PROOF).unwrap();
        let proof = AuditProof::try_from(audit_proof).unwrap();

        proof.validate(&tree_head, &leaf).unwrap();
    }
}
