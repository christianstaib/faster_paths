use crate::graphs::types::Weight;

use super::State;

pub struct BucketQueue {
    current_index: usize,
    num_elements: u32,
    buckets: Vec<Vec<State>>,
}

impl BucketQueue {
    pub fn new(max_edge_weight: Weight) -> BucketQueue {
        let buckets = vec![Vec::new(); max_edge_weight as usize + 1];
        BucketQueue {
            current_index: 0,
            num_elements: 0,
            buckets,
        }
    }

    pub fn push(&mut self, state: State) {
        let key_index = state.weight as usize % self.buckets.len();
        self.buckets[key_index].push(state);
        self.num_elements += 1;
    }

    pub fn pop(&mut self) -> Option<State> {
        for bucket_index in 0..self.buckets.len() {
            let key_index = (self.current_index + bucket_index) % self.buckets.len();
            if let Some(value) = self.buckets[key_index].pop() {
                self.current_index = key_index;
                self.num_elements -= 1;
                return Some(value);
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.num_elements == 0
    }
}
