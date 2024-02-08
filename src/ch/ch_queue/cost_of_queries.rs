use crate::graph::Graph;

use super::queue::PriorityTerm;

pub struct CostOfQueries {
    costs: Vec<i32>,
}

impl PriorityTerm for CostOfQueries {
    #[allow(unused_variables)]
    fn priority(&self, v: u32, graph: &Graph) -> i32 {
        *self.costs.get(v as usize).unwrap()
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, v: u32, graph: &Graph) {
        let v_cost = self.costs[v as usize] + 1;
        for neighbor in graph.neighborhood(v, 1) {
            self.costs[neighbor as usize] = std::cmp::max(self.costs[neighbor as usize], v_cost);
        }
    }
}

impl CostOfQueries {
    pub fn new(num_nodes: u32) -> Self {
        Self {
            costs: vec![0; num_nodes as usize],
        }
    }
}
