use std::cmp::Ordering;

use crate::graphs::VertexId;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ChPriorityElement {
    pub vertex: VertexId,
    pub priority: i32,
}

impl ChPriorityElement {
    pub fn new(priority: i32, vertex: VertexId) -> Self {
        Self { vertex, priority }
    }
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for ChPriorityElement {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.vertex.cmp(&other.vertex))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for ChPriorityElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
