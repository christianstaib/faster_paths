use crate::graphs::{graph::Graph, types::VertexId};

use super::queue::PriorityTerm;

pub struct CostOfQueries {
    costs: Vec<i32>,
}

impl PriorityTerm for CostOfQueries {
    #[allow(unused_variables)]
    fn priority(&self, vertex: VertexId, graph: &Graph) -> i32 {
        *self.costs.get(vertex as usize).unwrap()
    }

    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph) {
        let cost_of_vertex = self.costs[vertex as usize] + 1;

        for neighbor in graph.open_neighborhood(vertex, 1) {
            if cost_of_vertex > self.costs[neighbor as usize] {
                self.costs[neighbor as usize] = cost_of_vertex;
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
