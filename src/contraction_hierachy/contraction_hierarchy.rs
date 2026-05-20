use crate::{
    contraction_hierachy::edge::ContractionEdge,
    graph::{CsrGraph, GraphLike},
    types::Distance,
};
use serde::{Deserialize, Serialize};

/// Stores the upward and downward graphs of a contraction hierarchy.
///
/// Both graphs contain [`ContractionEdge`]s in upward direction, i.e. from lower
/// to higher contraction level.
#[derive(Serialize, Deserialize)]
pub struct ContractionHierarchy<D: Distance> {
    up_graph: CsrGraph<ContractionEdge<D>>,
    down_graph: CsrGraph<ContractionEdge<D>>,
}

impl<D: Distance> ContractionHierarchy<D> {
    pub fn new(
        up_graph: CsrGraph<ContractionEdge<D>>,
        down_graph: CsrGraph<ContractionEdge<D>>,
    ) -> Self {
        Self {
            up_graph,
            down_graph,
        }
    }

    /// Returns a reference to upward graph.
    pub fn up_graph(&self) -> &CsrGraph<ContractionEdge<D>> {
        &self.up_graph
    }

    /// Returns a reference to downward graph.
    pub fn down_graph(&self) -> &CsrGraph<ContractionEdge<D>> {
        &self.down_graph
    }

    /// Returns the number of vertices.
    pub fn num_vertices(&self) -> usize {
        std::cmp::max(self.up_graph.num_vertices(), self.down_graph.num_vertices())
    }
}
