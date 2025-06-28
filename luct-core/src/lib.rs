use crate::utils::base64::Base64;
use serde::{Deserialize, Serialize};
use url::Url;

pub(crate) mod utils;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CtLog {
    url: Url,
    log_id: Base64<Vec<u8>>,
    key: Base64<Vec<u8>>,
    mdd: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARGON2025H1: &str = "
url = \"https://ct.googleapis.com/logs/us1/argon2025h1/\"
log_id = \"TnWjJ1yaEMM4W2zU3z9S6x3w4I4bjWnAsfpksWKaOd8=\"
key = \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEIIKh+WdoqOTblJji4WiH5AltIDUzODyvFKrXCBjw/Rab0/98J4LUh7dOJEY7+66+yCNSICuqRAX+VPnV8R1Fmg==\"
mdd = 86400
    ";

    fn get_log() -> CtLog {
        toml::from_str(ARGON2025H1).unwrap()
    }

    #[test]
    fn ct_log_tom_parses() {
        let _ = get_log();
    }
}
