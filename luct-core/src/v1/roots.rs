use crate::{Certificate, v1::responses::GetRootsResponse};

impl From<&GetRootsResponse> for Vec<Certificate> {
    fn from(response: &GetRootsResponse) -> Self {
        response
            .certificates
            .iter()
            .filter_map(|cert| Certificate::from_der(&cert.0).ok())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Certificate, CertificateChain, cert::Fingerprint, tests::CERT_CHAIN_GOOGLE_COM,
        v1::responses::GetRootsResponse,
    };
    use std::collections::BTreeMap;

    const ARGON2025H5_GET_ROOTS_RESPONSE: &str =
        include_str!("../../../testdata/argon2025h2-get-roots.json");

    #[test]
    fn get_roots_response() {
        let response: GetRootsResponse =
            serde_json::from_str(ARGON2025H5_GET_ROOTS_RESPONSE).unwrap();

        let certs: Vec<Certificate> = (&response).into();
        // NOTE: One certificate in the get-roots response is not RFC 5280 compliant and
        // therefore can't be parsed with X509-certs
        assert_eq!(response.certificates.len(), 664);
        assert_eq!(certs.len(), 663);
    }

    #[test]
    fn validate_root_of_chain() {
        let response: GetRootsResponse =
            serde_json::from_str(ARGON2025H5_GET_ROOTS_RESPONSE).unwrap();

        let roots: Vec<Certificate> = (&response).into();
        let roots: BTreeMap<Fingerprint, Certificate> = roots
            .into_iter()
            .map(|cert| (cert.fingerprint_sha256(), cert))
            .collect();

        let cert = CertificateChain::from_pem_chain(CERT_CHAIN_GOOGLE_COM).unwrap();
        let root = roots.get(&cert.root().fingerprint_sha256()).unwrap();

        cert.verify_chain_against_root(root).unwrap();
    }
}
