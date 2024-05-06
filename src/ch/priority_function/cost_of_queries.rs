use super::PriorityFunction;
use crate::{
    ch::Shortcut,
    graphs::{graph_functions::neighbors, Graph, VertexId},
};

pub struct CostOfQueries {
    costs: Vec<i32>,
}

impl PriorityFunction for CostOfQueries {
    #[allow(unused_variables)]
    fn priority(&self, vertex: VertexId, graph: &dyn Graph, shortcuts: &Vec<Shortcut>) -> i32 {
        self.costs[vertex as usize]
    }

    fn update(&mut self, vertex: VertexId, graph: &dyn Graph) {
        self.costs[vertex as usize] += 1;

        for neighbor in neighbors(vertex, graph) {
            if self.costs[vertex as usize] > self.costs[neighbor as usize] {
                self.costs[neighbor as usize] = self.costs[vertex as usize];
            }
        }
    }

    fn initialize(&mut self, graph: &dyn Graph) {
        self.costs = vec![0; graph.number_of_vertices() as usize];
    }
}

impl Default for CostOfQueries {
    fn default() -> Self {
        Self::new()
    }
}

impl CostOfQueries {
    pub fn new() -> Self {
        Self { costs: Vec::new() }
    }
}
