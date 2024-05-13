use std::cmp::Ordering;

use crate::graphs::{VertexId, Weight};

pub mod bucket_queue;
pub mod heap_queue;
pub mod keyed_queue;
pub mod radix_queue;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DijkstraQueueElement {
    pub weight: Weight,
    pub vertex: VertexId,
}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for DijkstraQueueElement {
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
impl PartialOrd for DijkstraQueueElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl DijkstraQueueElement {
    pub fn new(weight: Weight, vertex: VertexId) -> DijkstraQueueElement {
        DijkstraQueueElement { weight, vertex }
    }
}

pub trait DijkstaQueue {
    fn push(&mut self, state: DijkstraQueueElement);
    fn pop(&mut self) -> Option<DijkstraQueueElement>;
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
}
