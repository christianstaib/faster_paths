use crate::{
    ch::contraction_helper::ShortcutSearchResult,
    graphs::{graph::Graph, types::VertexId},
};

use super::priority_term::PriorityTerm;

pub struct DeletedNeighbors {
    deleted_neighbors: Vec<u32>,
}

impl PriorityTerm for DeletedNeighbors {
    #[allow(unused_variables)]
    fn priority(
        &self,
        vertex: VertexId,
        graph: &Graph,
        shortcuts_results: &ShortcutSearchResult,
    ) -> i32 {
        self.deleted_neighbors[vertex as usize] as i32
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph) {
        graph
            .open_neighborhood(vertex, 1)
            .iter()
            .for_each(|&neighbor| self.deleted_neighbors[neighbor as usize] += 1);
    }
}

impl DeletedNeighbors {
    pub fn new(graph: &Graph) -> Self {
        Self {
            deleted_neighbors: vec![0; graph.number_of_vertices() as usize],
        }
    }
}
