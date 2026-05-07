use crate::{flattened_nested::FlattenedNested, graph::EdgeLike, types::VertexId};

/// Common interface for graphs.
pub trait GraphLike {
    type Edge: EdgeLike;

    /// Returns all outgoing edges of the given vertex.
    fn out_edges(&self, tail: VertexId) -> &[Self::Edge];

    /// Returns the number of vertices in the graph.
    fn num_vertices(&self) -> usize;

    /// Returns the number of edges in the graph.
    fn num_edges(&self) -> usize;

    /// Returns all edges in the graph.
    fn edges(&self) -> impl Iterator<Item = &Self::Edge> + '_ {
        (0..self.num_vertices())
            .map(|tail| VertexId::new(tail as u32))
            .flat_map(|tail| self.out_edges(tail).iter())
    }
}

impl<E: EdgeLike> GraphLike for FlattenedNested<E> {
    type Edge = E;

    fn out_edges(&self, tail: VertexId) -> &[Self::Edge] {
        self.nested(tail.as_usize())
    }

    fn num_vertices(&self) -> usize {
        self.num_nested()
    }

    fn num_edges(&self) -> usize {
        self.num_flat()
    }
}
