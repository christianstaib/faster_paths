use crate::{
    graph::{AdjacencyListGraph, Edge, EdgeLike, GraphLike},
    types::VertexId,
};

pub struct DirectionalAdjacencyListGraph<E: EdgeLike> {
    forward: AdjacencyListGraph<E>,
    reverse: AdjacencyListGraph<E>,
}

impl<E: EdgeLike> GraphLike for DirectionalAdjacencyListGraph<E> {
    type Edge = E;

    fn outgoing_edges(&self, tail: VertexId) -> &[Self::Edge] {
        self.forward.out_edges(tail)
    }

    fn num_vertices(&self) -> usize {
        self.forward.num_vertices()
    }

    fn num_edges(&self) -> usize {
        self.forward.num_edges()
    }
}

impl<E: EdgeLike> DirectionalAdjacencyListGraph<E> {
    /// Given a normal graph, build a `WorkingGraph` which is used during contraction.
    pub fn new() -> DirectionalAdjacencyListGraph<E> {
        Self {
            forward: AdjacencyListGraph::new(),
            reverse: AdjacencyListGraph::new(),
        }
    }

    pub fn forward_graph(&self) -> &AdjacencyListGraph<E> {
        &self.forward
    }

    /// Reverse incoming adjacency, stored as `vertex -> predecessor`.
    pub fn reverse_graph(&self) -> &AdjacencyListGraph<E> {
        &self.reverse
    }

    /// Inserts an edge into the graph and records its reverse adjacency for the head.
    pub fn add_edge(&mut self, edge: &E)
    where
        E: Copy,
    {
        // While self loops are not forbidden for contraction, they make it impossible to unpack a shortcut path containing them, as they create a cycle.
        if edge.tail() == edge.head() {
            return;
        }

        self.forward.add_edge(edge);
        self.reverse.add_edge(&edge.reversed());
    }

    /// Removes all out and in edges with head == vertex
    pub fn make_unreachable(&mut self, vertex: VertexId) {
        let create_edge = |from| Edge {
            tail: from,
            head: vertex,
        };

        for edge in self.forward.out_edges(vertex) {
            self.reverse.remove_edge(create_edge(edge.head())).unwrap();
        }

        for edge in self.reverse.out_edges(vertex) {
            self.forward.remove_edge(create_edge(edge.head())).unwrap();
        }
    }

    pub fn into_edge_lists(self) -> (Vec<E>, Vec<E>)
    where
        E: Copy,
    {
        (
            self.forward.all_edges().copied().collect(),
            self.reverse.all_edges().copied().collect(),
        )
    }
}
