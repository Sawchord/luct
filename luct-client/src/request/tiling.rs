use crate::{Client, ClientError, CtClient};
use luct_core::tiling::{Tile, TileId};
use url::Url;

impl<C: Client> CtClient<C> {
    pub async fn get_tile(&self, tile_id: TileId) -> Result<Tile, ClientError> {
        self.assert_v1()?;
        let url = self.get_url(&tile_id.as_url())?;

        let (status, response) = self.client.get_bin(&url, &[]).await?;
        self.check_status_binary(&url, status, &response)?;

        Ok(tile_id.with_data(response))
    }

    fn get_url(&self, path: &str) -> Result<Url, ClientError> {
        self.log
            .config()
            .tile_url()
            .as_ref()
            .map(|url| url.join(path).map_err(|_| ClientError::NonTilingLog))
            .ok_or(ClientError::NonTilingLog)
            .flatten()
    }
}
