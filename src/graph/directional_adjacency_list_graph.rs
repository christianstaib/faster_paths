use crate::{
    graph::{AdjacencyListGraph, CsrGraph, Edge, EdgeLike, GraphLike},
    types::Vertex,
};

/// A graph represented by two adjacency lists, allowing outgoing and incoming
/// edges to be queried.
///
/// Incoming edges are stored in reverse direction. For example, an edge
/// `(0 -> 1)` is stored in its original direction in `forward`, but as its
/// reverse edge `(1 -> 0)` in `reverse`.
pub struct DirectionalAdjacencyListGraph<E: EdgeLike> {
    forward: AdjacencyListGraph<E>,
    reverse: AdjacencyListGraph<E>,
}

impl<E: EdgeLike> GraphLike for DirectionalAdjacencyListGraph<E> {
    type Edge = E;

    fn outgoing_edges(&self, tail: Vertex) -> &[Self::Edge] {
        self.forward.outgoing_edges(tail)
    }

    fn num_vertices(&self) -> usize {
        self.forward.num_vertices()
    }

    fn num_edges(&self) -> usize {
        self.forward.num_edges()
    }
}

impl<E: EdgeLike> DirectionalAdjacencyListGraph<E> {
    /// Returns a new, empty graph.
    pub fn new() -> DirectionalAdjacencyListGraph<E> {
        Self {
            forward: AdjacencyListGraph::new(),
            reverse: AdjacencyListGraph::new(),
        }
    }

    /// Returns the forward graph, which contains all edges in their original direction.
    pub fn forward(&self) -> &AdjacencyListGraph<E> {
        &self.forward
    }

    /// Returns the reverse graph, which contains all edges in reversed direction.
    pub fn reverse(&self) -> &AdjacencyListGraph<E> {
        &self.reverse
    }

    /// Inserts an edge into the graph.
    ///
    /// The edge is stored in its original direction in the forward graph and in
    /// reversed direction in the reverse graph.
    pub fn add_edge(&mut self, edge: &E)
    where
        E: Copy,
    {
        self.forward.add_edge(edge);
        self.reverse.add_edge(&edge.reversed());
    }

    /// Removes all references to! `vertex`.
    pub fn make_unreachable(&mut self, vertex: Vertex) {
        let create_edge = |from| Edge {
            tail: from,
            head: vertex,
        };

        for edge in self.forward.outgoing_edges(vertex) {
            self.reverse.remove_edge(create_edge(edge.head())).unwrap();
        }

        for edge in self.reverse.outgoing_edges(vertex) {
            self.forward.remove_edge(create_edge(edge.head())).unwrap();
        }
    }

    /// Consumes the graph and returns its forward and reverse edge lists.
    pub fn into_csr_graphs(&self) -> (CsrGraph<E>, CsrGraph<E>)
    where
        E: Copy,
    {
        (
            CsrGraph::from_nested(self.forward.get_edges()),
            CsrGraph::from_nested(self.reverse.get_edges()),
        )
    }
}

impl<E: EdgeLike> Default for DirectionalAdjacencyListGraph<E> {
    fn default() -> Self {
        Self::new()
    }
}
