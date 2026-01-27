use crate::{store::Hashable, tree::HashOutput};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]

pub struct NodeKey {
    pub(crate) start: u64,
    pub(crate) end: u64,
}

impl NodeKey {
    pub fn leaf(idx: u64) -> Self {
        Self {
            start: idx,
            end: idx + 1,
        }
    }

    pub(crate) fn full_range(end: u64) -> Self {
        Self { start: 0, end }
    }

    pub(crate) fn size(&self) -> u64 {
        self.end - self.start
    }

    pub(crate) fn split_idx(&self) -> u64 {
        let size = self.size();
        size.next_power_of_two() >> 1
    }

    pub(crate) fn split(&self) -> (Self, Self) {
        let split = self.split_idx();
        let split = self.start + split;
        (
            Self {
                start: self.start,
                end: split,
            },
            Self {
                start: split,
                end: self.end,
            },
        )
    }

    pub(crate) fn merge(&self, other: &Self) -> Option<Self> {
        if self.end == other.start {
            Some(Self {
                start: self.start,
                end: other.end,
            })
        } else {
            None
        }
    }

    pub(crate) fn is_balanced(&self) -> bool {
        let diff = self.end - self.start;
        diff.is_power_of_two()
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.start + 1 == self.end
    }
}

impl PartialOrd for NodeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.end.cmp(&other.end) {
            Ordering::Equal => {}
            ord => return ord,
        }
        self.start.cmp(&other.start)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub(crate) left: HashOutput,
    pub(crate) right: HashOutput,
}

impl Hashable for Node {
    fn hash(&self) -> HashOutput {
        let mut hash = Sha256::new();
        hash.update([1]);
        hash.update(self.left);
        hash.update(self.right);
        hash.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_key_split() {
        let node_key = NodeKey { start: 0, end: 6 };
        let (left, right) = node_key.split();
        assert_eq!(left, NodeKey { start: 0, end: 4 });
        assert_eq!(right, NodeKey { start: 4, end: 6 });

        let node_key = NodeKey { start: 0, end: 8 };
        let (left, right) = node_key.split();
        assert_eq!(left, NodeKey { start: 0, end: 4 });
        assert_eq!(right, NodeKey { start: 4, end: 8 });
    }
}
