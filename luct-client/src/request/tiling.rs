use crate::{Client, ClientError, CtClient};
use luct_core::tiling::{Tile, TileId};
use url::Url;

impl<C: Client> CtClient<C> {
    pub async fn get_tile(&self, mut tile_id: TileId) -> Result<Tile, ClientError> {
        self.assert_v1()?;

        let url = self.get_url(&tile_id.as_url())?;
        let (mut status, mut response) = self.client.get_bin(&url, &[]).await?;

        // If the partial tile can't be found, we retry with the full tile
        if status == 404 && tile_id.is_partial() {
            tile_id = tile_id.into_unpartial();
            let url = self.get_url(&tile_id.as_url())?;
            (status, response) = self.client.get_bin(&url, &[]).await?;
        };

        self.check_status_binary(&url, status, &response)?;

        tile_id
            .with_data(response)
            .ok_or(ClientError::MalformedTile)
    }

    // TODO: Get Data tile

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
