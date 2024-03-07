use crate::graphs::graph::Graph;

use super::priority_term::PriorityFunction;

pub struct SearchSpaceSize {}

impl SearchSpaceSize {
    #[allow(unused_variables)]
    pub fn new(graph: &Graph) -> Self {
        Self {}
    }
}

impl PriorityFunction for SearchSpaceSize {
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
    fn update(
        &mut self,
        vertex: crate::graphs::types::VertexId,
        graph: &crate::graphs::graph::Graph,
    ) {
    }
}
