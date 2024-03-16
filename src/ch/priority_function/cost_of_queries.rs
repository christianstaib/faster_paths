use crate::{
    ch::ShortcutSearchResult,
    graphs::{graph::Graph, VertexId},
};

use super::PriorityFunction;

pub struct CostOfQueries {
    costs: Vec<i32>,
}

impl PriorityFunction for CostOfQueries {
    #[allow(unused_variables)]
    fn priority(
        &self,
        vertex: VertexId,
        graph: &Graph,
        shortcuts_results: &ShortcutSearchResult,
    ) -> i32 {
        self.costs[vertex as usize]
    }

    fn update(&mut self, vertex: VertexId, graph: &Graph) {
        self.costs[vertex as usize] += 1;

        for neighbor in graph.open_neighborhood(vertex, 1) {
            if self.costs[vertex as usize] > self.costs[neighbor as usize] {
                self.costs[neighbor as usize] = self.costs[vertex as usize];
            }
        }
    }
}

impl CostOfQueries {
    pub fn new(graph: &Graph) -> Self {
        Self {
            costs: vec![0; graph.number_of_vertices() as usize],
        }
    }
}
