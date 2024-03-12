use crate::{
    ch::ShortcutSearchResult,
    graphs::{graph::Graph, types::VertexId},
};

use super::priority_function::PriorityFunction;

pub struct EdgeDifference {}

impl EdgeDifference {
    #[allow(unused_variables)]
    pub fn new(graph: &Graph) -> Self {
        Self {}
    }
}

impl PriorityFunction for EdgeDifference {
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
    fn update(&mut self, vertex: VertexId, graph: &Graph) {}
}
