use crate::graphs::graph::Graph;

use super::queue::PriorityTerm;

pub struct EdgeDifference {}

impl EdgeDifference {
    #[allow(unused_variables)]
    pub fn new(graph: &Graph) -> Self {
        Self {}
    }
}

impl PriorityTerm for EdgeDifference {
    #[allow(unused_variables)]
    fn priority(
        &self,
        vertex: crate::graphs::types::VertexId,
        graph: &crate::graphs::graph::Graph,
        shortcuts_results: &crate::ch::contraction_helper::ShortcutSearchResult,
    ) -> i32 {
        shortcuts_results.edge_difference
    }

    #[allow(unused_variables)]
    fn update_before_contraction(
        &mut self,
        vertex: crate::graphs::types::VertexId,
        graph: &crate::graphs::graph::Graph,
    ) {
    }
}
