use crate::{ClientError, CtClient};
use std::fmt::Debug;
use url::Url;

impl<C: Debug> CtClient<C> {
    pub(crate) fn check_status(
        &self,
        url: &Url,
        status: u16,
        response: &str,
    ) -> Result<(), ClientError> {
        if status != 200 {
            let err = ClientError::ResponseError {
                url: url.to_string(),
                code: status,
                msg: response.to_string(),
            };
            tracing::error!("Endpoint {} returned and error: {}", url, err);

            Err(err)
        } else {
            Ok(())
        }
    }

    pub(crate) fn check_status_binary(
        &self,
        url: &Url,
        status: u16,
        response: &[u8],
    ) -> Result<(), ClientError> {
        self.check_status(url, status, &String::from_utf8_lossy(response))
    }
}
