use std::collections::BinaryHeap;

use super::State;

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

    pub fn push(&mut self, state: State) {
        self.queue.push(state)
    }

    pub fn pop(&mut self) -> Option<State> {
        self.queue.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
