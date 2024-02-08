use std::collections::HashSet;

use rayon::prelude::{ParallelBridge, ParallelIterator};

use crate::graph::Graph;

use super::queue::PriorityTerm;

pub struct DeletedNeighbors {
    deleted: Vec<bool>,
}

impl PriorityTerm for DeletedNeighbors {
    fn priority(&self, v: u32, graph: &Graph) -> i32 {
        let neighbors: HashSet<_> = graph.in_edges[v as usize]
            .iter()
            .map(|edge| edge.tail)
            .collect();
        neighbors
            .iter()
            .par_bridge()
            .filter(|&&neighbor| self.deleted[neighbor as usize])
            .count() as i32
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, v: u32, graph: &Graph) {
        self.deleted[v as usize] = true;
    }
}

impl DeletedNeighbors {
    pub fn new(num_nodes: u32) -> Self {
        Self {
            deleted: vec![false; num_nodes as usize],
        }
    }
}
