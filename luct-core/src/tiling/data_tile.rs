use crate::tiling::index_to_url;
use std::num::NonZeroU8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTileId {
    index: u64,
    partial: Option<NonZeroU8>,
}

impl DataTileId {
    pub fn as_url(&self) -> String {
        let index_url = index_to_url(self.index);

        match self.partial {
            Some(partial) => format!("/tile/data/{}.p/{}", index_url, partial),
            None => format!("/tile/data/{}", index_url),
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
        assert_eq!(&data_tile_id(1, None).as_url(), "/tile/data/001");
        assert_eq!(
            &data_tile_id(10987654321, None).as_url(),
            "/tile/data/x010/x987/x654/321"
        );
        assert_eq!(
            &data_tile_id(1234, Some(128)).as_url(),
            "/tile/data/x001/234.p/128"
        );
    }

    fn data_tile_id(index: u64, partial: Option<u8>) -> DataTileId {
        DataTileId {
            index,
            partial: partial.map(|partial| NonZeroU8::new(partial).unwrap()),
        }
    }
}
