use crate::{tiling::index_to_url, tree::NodeKey};
use std::num::NonZeroU8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileId {
    level: u8,
    index: u64,
    partial: Option<NonZeroU8>,
}

impl TileId {
    pub fn from_node_key(key: &NodeKey, tree_height: u64) -> Option<Self> {
        let level: u8 = (key.size().next_power_of_two() / 8).try_into().unwrap();

        let steps: u64 = (level as u64).pow(8 * level as u32);

        let index = key.start / steps;

        let tile_end = (index + 1) * steps;

        let partial = if tile_end < tree_height {
            None
        } else {
            let partial: u8 = ((tree_height % steps) >> (8 * level)).try_into().unwrap();
            Some(NonZeroU8::new(partial).unwrap())
        };

        Some(Self {
            level,
            index,
            partial,
        })
    }

    pub fn as_url(&self) -> String {
        let index_url = index_to_url(self.index);

        match self.partial {
            Some(partial) => format!("/tile/{}/{}.p/{}", self.level, index_url, partial),
            None => format!("/tile/{}/{}", self.level, index_url),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tile {
    id: TileId,
    data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_tile_id() {
        assert_eq!(
            TileId::from_node_key(
                &NodeKey {
                    start: 1 << 16,
                    end: 70000
                },
                70000
            )
            .unwrap(),
            TileId {
                level: 1,
                index: 1,
                partial: Some(NonZeroU8::new(17).unwrap())
            }
        );
    }
}
