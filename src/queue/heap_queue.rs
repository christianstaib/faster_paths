use std::collections::BinaryHeap;

use super::{DijkstaQueue, State};

#[derive(Clone)]
pub struct HeapQueue {
    queue: BinaryHeap<State>,
}

impl HeapQueue {
    pub fn new() -> HeapQueue {
        HeapQueue {
            queue: BinaryHeap::new(),
        }
    }
}

impl DijkstaQueue for HeapQueue {
    fn push(&mut self, state: State) {
        self.queue.push(state)
    }

    fn pop(&mut self) -> Option<State> {
        self.queue.pop()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
