use std::cmp::Ordering;

use crate::graphs::types::{VertexId, Weight};

pub mod bucket_queue;
pub mod heap_queue;
pub mod radix_queue;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct State {
    pub weight: Weight,
    pub vertex: VertexId,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for State {
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
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl State {
    pub fn new(weight: Weight, vertex: VertexId) -> State {
        State { weight, vertex }
    }
}
