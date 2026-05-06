use crate::{edge_like::EdgeLike, types::VertexId};

/// Common interface for graphs.
pub trait GraphLike {
    type Edge: EdgeLike;

    /// Returns all outgoing edges of the given vertex.
    fn out_edges(&self, tail: VertexId) -> &[Self::Edge];

    /// Returns the number of vertices in the graph.
    fn num_vertices(&self) -> usize;

    /// Returns the number of edges in the graph.
    fn num_edges(&self) -> usize;
}
