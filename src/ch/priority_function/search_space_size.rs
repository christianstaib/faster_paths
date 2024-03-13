use crate::{ch::ShortcutSearchResult, graphs::graph::Graph};

use super::PriorityFunction;

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
        shortcuts_results: &ShortcutSearchResult,
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
