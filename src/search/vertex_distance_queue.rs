use std::{cmp::Reverse, collections::BinaryHeap};

use radix_heap::RadixHeapMap;

use crate::graphs::{VertexId, Weight};

pub trait VertexDistanceQueue {
    fn clear(&mut self);

    fn insert(&mut self, vertex: VertexId, distance: Weight);

    fn pop(&mut self) -> Option<VertexId>;
}

pub struct VertexDistanceQueueRadixHeap {
    heap: RadixHeapMap<i64, VertexId>,
}

impl VertexDistanceQueueRadixHeap {
    pub fn new() -> Self {
        VertexDistanceQueueRadixHeap {
            heap: RadixHeapMap::new(),
        }
    }
}

impl VertexDistanceQueue for VertexDistanceQueueRadixHeap {
    fn clear(&mut self) {
        self.heap.clear();
    }

    fn insert(&mut self, vertex: VertexId, distance: Weight) {
        self.heap.push(-(distance as i64), vertex);
    }

    fn pop(&mut self) -> Option<VertexId> {
        self.heap.pop().map(|(_negative_distance, vertex)| vertex)
    }
}

pub struct VertexDistanceQueueBinaryHeap {
    heap: BinaryHeap<Reverse<(Weight, VertexId)>>,
}

impl VertexDistanceQueueBinaryHeap {
    pub fn new() -> Self {
        VertexDistanceQueueBinaryHeap {
            heap: BinaryHeap::new(),
        }
    }
}

impl VertexDistanceQueue for VertexDistanceQueueBinaryHeap {
    fn clear(&mut self) {
        self.heap.clear();
    }

    fn insert(&mut self, vertex: VertexId, distance: Weight) {
        self.heap.push(Reverse((distance, vertex)));
    }

    fn pop(&mut self) -> Option<VertexId> {
        let Reverse((_distance, vertex)) = self.heap.pop()?;

        Some(vertex)
    }
}
