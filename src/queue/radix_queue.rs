use radix_heap::RadixHeapMap;

use super::{DijkstaQueue, DijkstraQueueElement};

#[derive(Clone)]
pub struct RadixQueue {
    heap: RadixHeapMap<i32, u32>,
}

impl Default for RadixQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl RadixQueue {
    pub fn new() -> RadixQueue {
        RadixQueue {
            heap: RadixHeapMap::new(),
        }
    }
}
impl DijkstaQueue for RadixQueue {
    fn push(&mut self, state: DijkstraQueueElement) {
        self.heap.push(-(state.weight as i32), state.vertex);
    }

    fn pop(&mut self) -> Option<DijkstraQueueElement> {
        let (negative_weight, vertex) = self.heap.pop()?;
        Some(DijkstraQueueElement {
            weight: -negative_weight as u32,
            vertex,
        })
    }

    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    fn clear(&mut self) {
        self.heap.clear();
    }
}
