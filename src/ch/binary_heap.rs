use std::cmp::Ordering;

use crate::graphs::types::VertexId;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct MinimumItem {
    pub priority: u32,
    pub vertex: VertexId,
}

impl MinimumItem {
    pub fn new(priority: u32, vertex: VertexId) -> Self {
        Self { priority, vertex }
    }
}

impl Ord for MinimumItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on priority.
        // In case of a tie we compare vertices - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.vertex.cmp(&other.vertex))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for MinimumItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
