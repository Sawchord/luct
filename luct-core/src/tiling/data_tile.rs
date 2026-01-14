use crate::tiling::index_to_url;
use std::num::NonZeroU8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTileId {
    index: u64,
    partial: Option<NonZeroU8>,
}

impl DataTileId {
    /// Returns the [`DataTileId`] of the tile, which contains the `index`.
    ///
    /// The `tree_height` is used to calculate, wether the tile in question should be partial or not.
    pub fn from_index(index: u64, tree_height: u64) -> Option<Self> {
        let tile_width = 256;

        // Compute the index of the tile, that should contain the node
        let index = index / tile_width;

        // Check if we need to fetch a partial tile, and if so, compute it's size
        let tile_end = (index + 1) * tile_width;
        let partial = if tile_end < tree_height {
            None
        } else {
            let partial: u8 = (tree_height % tile_width).try_into().unwrap();
            Some(NonZeroU8::new(partial).unwrap())
        };

        Some(Self { index, partial })
    }

    pub fn as_url(&self) -> String {
        let index_url = index_to_url(self.index);

        match self.partial {
            Some(partial) => format!("tile/data/{}.p/{}", index_url, partial),
            None => format!("tile/data/{}", index_url),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTile {
    id: DataTileId,
    data: Vec<u8>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn as_url() {
        assert_eq!(&data_tile_id(1, None).as_url(), "tile/data/001");
        assert_eq!(
            &data_tile_id(10987654321, None).as_url(),
            "tile/data/x010/x987/x654/321"
        );
        assert_eq!(
            &data_tile_id(1234, Some(128)).as_url(),
            "tile/data/x001/234.p/128"
        );
    }

    #[test]
    fn into_data_tile_id() {
        assert_eq!(
            DataTileId::from_index(4, 70000).unwrap(),
            data_tile_id(0, None)
        );
        assert_eq!(
            DataTileId::from_index(270, 70000).unwrap(),
            data_tile_id(1, None)
        );
        assert_eq!(
            DataTileId::from_index(69950, 70000).unwrap(),
            data_tile_id(273, Some(112))
        );
    }

    fn data_tile_id(index: u64, partial: Option<u8>) -> DataTileId {
        DataTileId {
            index,
            partial: partial.map(|partial| NonZeroU8::new(partial).unwrap()),
        }
    }
}
