use crate::graphs::{graph::Graph, types::VertexId};

use super::queue::PriorityTerm;

pub struct DeletedNeighbors {
    deleted: Vec<bool>,
}

impl PriorityTerm for DeletedNeighbors {
    fn priority(&self, vertex: VertexId, graph: &Graph) -> i32 {
        let neighbors = graph.open_neighborhood(vertex, 1);
        neighbors
            .iter()
            .filter(|&&neighbor| self.deleted[neighbor as usize])
            .count() as i32
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph) {
        self.deleted[vertex as usize] = true;
    }
}

impl DeletedNeighbors {
    pub fn new(num_nodes: u32) -> Self {
        Self {
            deleted: vec![false; num_nodes as usize],
        }
    }
}
