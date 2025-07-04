#[derive(Debug, Clone)]
pub(crate) struct Tree<N, L> {
    nodes: N,
    leafs: L,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TreeHead {
    tree_size: u64,
    head: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub(crate) struct NodeKey {
    start: usize,
    end: usize,
}

impl PartialOrd for NodeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NodeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.end.cmp(&other.end) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.start.cmp(&other.start)
    }
}
