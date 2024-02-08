use super::heap_queue::State;

pub struct BucketQueue {
    current_index: usize,
    buckets: Vec<Vec<State>>,
}

impl Default for BucketQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl BucketQueue {
    pub fn new() -> BucketQueue {
        let buckets = vec![Vec::new(); 30_001];
        BucketQueue {
            current_index: 0,
            buckets,
        }
    }

    pub fn insert(&mut self, key: u32, value: u32) {
        let state = State { key, value };
        let key_index = key as usize % self.buckets.len();
        self.buckets[key_index].push(state)
    }

    pub fn pop(&mut self) -> Option<State> {
        for bucket_index in 0..self.buckets.len() {
            let key_index = (self.current_index + bucket_index) % self.buckets.len();
            if let Some(value) = self.buckets[key_index].pop() {
                self.current_index = key_index;
                return Some(value);
            }
        }
        None
    }
}
