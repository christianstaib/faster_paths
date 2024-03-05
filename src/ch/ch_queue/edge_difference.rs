use crate::{
    ch::contraction_helper::ShortcutSearchResult,
    graphs::{graph::Graph, types::VertexId},
};

use super::priority_term::PriorityTerm;

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
        vertex: VertexId,
        graph: &Graph,
        shortcuts_results: &ShortcutSearchResult,
    ) -> i32 {
        shortcuts_results.edge_difference
    }

    #[allow(unused_variables)]
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph) {}
}
