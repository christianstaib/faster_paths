use ahash::{HashSet, HashSetExt};

use super::PriorityFunction;
use crate::{
    ch::Shortcut,
    graphs::{Graph, VertexId},
};

pub struct HittingSet {
    hitting_set: HashSet<VertexId>,
}

impl Default for HittingSet {
    fn default() -> Self {
        Self::new()
    }
}

impl HittingSet {
    pub fn new() -> Self {
        HittingSet {
            hitting_set: HashSet::new(),
        }
    }
}

impl PriorityFunction for HittingSet {
    fn priority(&self, vertex: u32, _graph: &Box<dyn Graph>, _shortcuts: &Vec<Shortcut>) -> i32 {
        if self.hitting_set.contains(&vertex) {
            return i32::MAX / 2;
        }
        0
    }

    fn update(&mut self, _vertex: u32, _graph: &Box<dyn Graph>) {}

    fn initialize(&mut self, _graph: &Box<dyn Graph>) {
        todo!()
    }
}
