use std::collections::BinaryHeap;

use super::{DijkstaQueue, DijkstraQueueElement};

#[derive(Clone)]
pub struct HeapQueue {
    queue: BinaryHeap<DijkstraQueueElement>,
}

impl Default for HeapQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl HeapQueue {
    pub fn new() -> HeapQueue {
        HeapQueue {
            queue: BinaryHeap::new(),
        }
    }
}

impl DijkstaQueue for HeapQueue {
    fn push(&mut self, state: DijkstraQueueElement) {
        self.queue.push(state)
    }

    fn pop(&mut self) -> Option<DijkstraQueueElement> {
        self.queue.pop()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn clear(&mut self) {
        self.queue.clear();
    }
}
