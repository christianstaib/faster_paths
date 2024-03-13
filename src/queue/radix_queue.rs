use radix_heap::RadixHeapMap;

use super::{DijkstaQueue, State};

#[derive(Clone)]
pub struct RadixQueue {
    heap: RadixHeapMap<i32, u32>,
}

impl RadixQueue {
    pub fn new() -> RadixQueue {
        RadixQueue {
            heap: RadixHeapMap::new(),
        }
    }
}
impl DijkstaQueue for RadixQueue {
    fn push(&mut self, state: State) {
        self.heap.push(-(state.weight as i32), state.vertex);
    }

    fn pop(&mut self) -> Option<State> {
        let (negative_weight, vertex) = self.heap.pop()?;
        Some(State {
            weight: -negative_weight as u32,
            vertex,
        })
    }

    fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
