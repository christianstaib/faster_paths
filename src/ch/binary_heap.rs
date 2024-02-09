use std::cmp::Ordering;

use crate::types::{VertexId, Weight};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MinimumItem {
    pub weight: Weight,
    pub vertex: VertexId,
}

impl MinimumItem {
    pub fn new(priority: u32, item: u32) -> Self {
        Self {
            weight: priority,
            vertex: item,
        }
    }
}

impl Ord for MinimumItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .weight
            .cmp(&self.weight)
            .then_with(|| self.vertex.cmp(&other.vertex))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for MinimumItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
