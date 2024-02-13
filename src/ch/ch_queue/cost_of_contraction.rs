use crate::{
    ch::contraction_helper::ContractionHelper,
    graphs::{graph::Graph, types::VertexId},
};

use super::queue::PriorityTerm;

pub struct CostOfContraction {}

impl CostOfContraction {
    pub fn new(_graph: &Graph) -> Self {
        Self {}
    }
}

impl PriorityTerm for CostOfContraction {
    fn priority(&self, vertex: VertexId, graph: &Graph) -> i32 {
        let contraction_helper = ContractionHelper::new(&graph, 10);
        contraction_helper.wittness_search_space(vertex)
    }

    fn update_before_contraction(&mut self, _vertex: VertexId, _graph: &Graph) {
        // todo!()
    }
}
