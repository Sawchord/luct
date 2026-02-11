mod checkpoint;
mod data_tile;
mod tile;

use crate::tree::ProofGenerationError;
pub use checkpoint::{Checkpoint, ParseCheckpointError};
pub use data_tile::{DataTile, DataTileId};
use itertools::Itertools;
use thiserror::Error;
pub use tile::{Tile, TileId};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TilingError {
    #[error("Can not fetch tiles from non tiling log")]
    NonTilingLog,

    #[error("The tile that was returned by the log is malformed")]
    MalformedTile,

    #[error("The SCT has no leaf index")]
    LeafIndexMissing,

    #[error("Failed to generate audit proof: {0}")]
    AuditProofGenerationError(ProofGenerationError),

    #[error("Failed to generate consistency proof: {0}")]
    ConsistencyProofGenerationError(ProofGenerationError),
}

/// Turn an index into a url as specified in the tiling spec, i.e. "1234067" to "x001/x234/067"
fn index_to_url(idx: u64) -> String {
    let idx = idx.to_string();

    let leading_zeros = (3 - idx.len() % 3) % 3;

    let num_segments = (idx.len() + leading_zeros) / 3;

    std::iter::repeat_n('0', leading_zeros)
        .chain(idx.chars())
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect::<String>())
        .enumerate()
        .map(|(idx, segment)| {
            if idx != num_segments - 1 {
                format!("x{segment}")
            } else {
                segment
            }
        })
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_to_url() {
        // Example from the spec
        assert_eq!(index_to_url(1234067), "x001/x234/067");

        assert_eq!(index_to_url(0), "000");
        assert_eq!(index_to_url(1), "001");
        assert_eq!(index_to_url(1000), "x001/000");
        assert_eq!(index_to_url(1001), "x001/001");

        assert_eq!(index_to_url(87654321), "x087/x654/321");
        assert_eq!(index_to_url(987654321), "x987/x654/321");
        assert_eq!(index_to_url(1987654321), "x001/x987/x654/321");
    }
}
