use crate::{
    ch::Shortcut,
    graphs::{graph_functions::neighbors, Graph, VertexId},
};

use super::PriorityFunction;

pub struct DeletedNeighbors {
    deleted_neighbors: Vec<u32>,
}

impl PriorityFunction for DeletedNeighbors {
    fn priority(
        &self,
        vertex: VertexId,
        _graph: &Box<dyn Graph>,
        _shortcuts: &Vec<Shortcut>,
    ) -> i32 {
        self.deleted_neighbors[vertex as usize] as i32
    }

    fn update(&mut self, _vertex: VertexId, _graph: &Box<dyn Graph>) {
        neighbors(_vertex, &**_graph)
            .iter()
            .for_each(|&neighbor| self.deleted_neighbors[neighbor as usize] += 1);
    }

    fn initialize(&mut self, graph: &Box<dyn Graph>) {
        self.deleted_neighbors = vec![0; graph.number_of_vertices() as usize];
    }
}

impl Default for DeletedNeighbors {
    fn default() -> Self {
        Self::new()
    }
}

impl DeletedNeighbors {
    pub fn new() -> Self {
        Self {
            deleted_neighbors: Vec::new(),
        }
    }
}
