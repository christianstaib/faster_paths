use std::cmp::Reverse;

use keyed_priority_queue::{Entry, KeyedPriorityQueue};

use super::{DijkstaQueue, DijkstraQueueElement};
use crate::graphs::{VertexId, Weight};

#[derive(Clone)]
pub struct KeyedQueue {
    queue: KeyedPriorityQueue<VertexId, Reverse<Weight>>,
}

impl Default for KeyedQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyedQueue {
    pub fn new() -> KeyedQueue {
        KeyedQueue {
            queue: KeyedPriorityQueue::new(),
        }
    }
}
impl DijkstaQueue for KeyedQueue {
    fn push(&mut self, state: DijkstraQueueElement) {
        match self.queue.entry(state.vertex) {
            Entry::Vacant(entry) => {
                entry.set_priority(Reverse(state.weight));
            }
            Entry::Occupied(entry) => {
                if Reverse(state.weight) < *entry.get_priority() {
                    entry.set_priority(Reverse(state.weight));
                }
            }
        };
    }

    fn pop(&mut self) -> Option<DijkstraQueueElement> {
        let (vertex, Reverse(weight)) = self.queue.pop()?;
        Some(DijkstraQueueElement { weight, vertex })
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn clear(&mut self) {
        self.queue.clear();
    }
}
