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
    use crate::{Certificate, v1::responses::GetRootsResponse};

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
}
