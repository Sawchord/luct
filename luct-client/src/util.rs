use crate::{ClientError, CtClient};
use url::Url;

impl<C> CtClient<C> {
    pub(crate) fn check_status(
        &self,
        url: &Url,
        status: u16,
        response: &str,
    ) -> Result<(), ClientError> {
        if status != 200 {
            return Err(ClientError::ResponseError {
                url: url.to_string(),
                code: status,
                msg: response.to_string(),
            });
        }

        Ok(())
    }
}
