use crate::{Client, ClientError, CtClient};
use luct_core::{
    tiling::{Checkpoint, Tile, TileId},
    v1::SignedTreeHead,
};
use url::Url;

impl<C: Client> CtClient<C> {
    #[tracing::instrument(level = "trace")]
    pub async fn get_checkpoint(&self) -> Result<SignedTreeHead, ClientError> {
        self.assert_v1()?;
        let url = self.get_url("checkpoint")?;

        // Fetch and parse checkpoint
        let (status, response) = self.client.get(&url, &[]).await?;
        self.check_status(&url, status, &response)?;
        let checkpoint = Checkpoint::parse_checkpoint(&response)?;

        // Validate checkpoint against key
        let sth = self
            .log
            .validate_checkpoint(&checkpoint)
            .map_err(|err| ClientError::SignatureValidationFailed("checkpoint STH", err))?;

        tracing::debug!(
            "fetched and validated checkpoint: {:?} from url {}",
            sth,
            url
        );

        Ok(sth)
    }

    #[tracing::instrument(level = "trace")]
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

        tracing::trace!("fetched tile {:?}, from url: {}", tile_id, url);

        tile_id
            .with_data(response)
            .ok_or(ClientError::MalformedTile)
    }

    // TODO: Get Data tile
    // TODO: Get issuer

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
#[cfg(all(test, feature = "reqwest"))]
mod tests {
    use super::*;
    use crate::reqwest::ReqwestClient;
    use luct_core::{CtLogConfig, tree::NodeKey};

    const ARCHE2026H1: &str = "{
          \"description\": \"Google 'Arche2026h1' log\",
          \"key\": \"MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEZ+3YKoZTMruov4cmlImbk4MckBNzEdCyMuHlwGgJ8BUrzFLlR5U0619xDDXIXespkpBgCNVQAkhMTTXakM6KMg==\",
          \"url\": \"https://arche2026h1.staging.ct.transparency.dev/\",
          \"tile_url\": \"https://storage.googleapis.com/static-ct-staging-arche2026h1-bucket/\",
          \"mmd\": 60
        }";

    #[test]
    fn get_url() {
        let client = get_client();
        let url = client
            .get_url(
                &TileId::from_node_key(&NodeKey::leaf(1), 1000)
                    .unwrap()
                    .as_url(),
            )
            .unwrap();

        assert_eq!(
            url.to_string(),
            "https://storage.googleapis.com/static-ct-staging-arche2026h1-bucket/tile/0/000"
        )
    }

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn get_checkpoint() {
        let client = get_client();
        let _ = client.get_checkpoint().await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Makes an HTTP call, for manual testing only"]
    async fn get_tile() {
        let client = get_client();

        let _ = client
            .get_tile(TileId::from_node_key(&NodeKey::leaf(1), 1000).unwrap())
            .await
            .unwrap();
    }

    fn get_client() -> CtClient<ReqwestClient> {
        let config: CtLogConfig = serde_json::from_str(ARCHE2026H1).unwrap();
        let client = ReqwestClient::new();
        CtClient::new(config, client)
    }
}
