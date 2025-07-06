use crate::{
    tree::{ConsistencyProof, TreeHead},
    v1::{responses::GetSthConsistencyResponse, sth::TreeHeadSignature},
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        CertificateChain,
        tests::{
            ARGON2025H1_STH2806, ARGON2025H1_STH2906, CERT_CHAIN_GOOGLE_COM,
            GOOGLE_STH_CONSISTENCY_PROOF, get_log_argon2025h2,
        },
        v1::SthResponse,
    };

    #[test]
    fn validate_sth_consistency() {
        let old_sth: SthResponse = serde_json::from_str(ARGON2025H1_STH2806).unwrap();
        let old_sth = TreeHeadSignature::try_from(&old_sth).unwrap();
        let old_tree_head = TreeHead::from(old_sth);

        let new_sth: SthResponse = serde_json::from_str(ARGON2025H1_STH2906).unwrap();
        let new_sth = TreeHeadSignature::try_from(&new_sth).unwrap();
        let new_tree_head = TreeHead::from(new_sth);

        let proof: GetSthConsistencyResponse =
            serde_json::from_str(GOOGLE_STH_CONSISTENCY_PROOF).unwrap();
        let proof = ConsistencyProof::try_from(proof).unwrap();
        assert!(proof.validate(&old_tree_head, &new_tree_head));
    }

    #[test]
    fn audit_sct() {
        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let scts = cert.cert().extract_scts_v1().unwrap();

        let log = get_log_argon2025h2();
        assert_eq!(log.log_id_v1(), scts[0].log_id());

        //let leaf = cert.as_leaf_v1(&scts[0], true).unwrap();
        //let hash = leaf.hash().unwrap();

        // TODO: Validate the audit proof against the STH
    }
}
