use crate::{ClientError, CtClient};
use luct_core::Version;
use url::Url;

impl<C> CtClient<C> {
    pub(crate) fn get_full_v1_url(&self) -> Url {
        let base_url = self
            .config
            .fetch_url
            .as_ref()
            .unwrap_or(self.config.log.url());

        base_url.join("ct/v1/").unwrap()
    }

    pub(crate) fn assert_v1(&self) -> Result<(), ClientError> {
        match self.config.log.version() {
            Version::V1 => Ok(()),
            #[allow(unreachable_patterns)]
            _ => Err(ClientError::UnsupportedVersion),
        }
    }

    pub(crate) fn check_status(&self, status: u16, response: &str) -> Result<(), ClientError> {
        if status != 200 {
            return Err(ClientError::ResponseError {
                code: status,
                msg: response.to_string(),
            });
        }

        Ok(())
    }
}
