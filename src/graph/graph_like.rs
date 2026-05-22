use crate::{graph::EdgeLike, types::Vertex};

/// Common interface for a directed, weighted graph.
///
/// Vertices are of type [`VertexId`]. Edges are exposed through outgoing
/// adjacency lists, which can be queried by their tail.
pub trait GraphLike {
    type Edge: EdgeLike;

    /// Returns the number of vertices in the graph.
    fn num_vertices(&self) -> usize;

    /// Returns the number of edges in the graph.
    fn num_edges(&self) -> usize;

    /// Returns a slice of all edges outgoing from `tail`.
    fn outgoing_edges(&self, tail: Vertex) -> &[Self::Edge];

    /// Returns an iterator over all edges in the graph.
    fn all_edges(&self) -> impl Iterator<Item = &Self::Edge> + '_ {
        (0..self.num_vertices() as u32)
            .map(Vertex::new)
            .flat_map(|tail| self.outgoing_edges(tail).iter())
    }
}
