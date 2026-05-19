use crate::{
    graph::{Edge, EdgeLike, Graph, GraphLike},
    types::VertexId,
};

pub(super) struct WorkingGraph<E: EdgeLike> {
    outgoing: Graph<E>,
    incoming: Graph<E>,
}

impl<E: EdgeLike> WorkingGraph<E> {
    pub(super) fn num_vertices(&self) -> usize {
        self.outgoing.num_vertices()
    }

    pub(super) fn outgoing(&self) -> &Graph<E> {
        &self.outgoing
    }

    /// Reverse incoming adjacency, stored as `vertex -> predecessor`.
    pub(super) fn incoming(&self) -> &Graph<E> {
        &self.incoming
    }
}

impl<E: EdgeLike> WorkingGraph<E> {
    /// Given a normal graph, build a `WorkingGraph` which is used during contraction.
    pub(super) fn new() -> WorkingGraph<E> {
        Self {
            outgoing: Graph::new(),
            incoming: Graph::new(),
        }
    }

    /// Inserts an edge into the graph and records its reverse adjacency for the head.
    pub(super) fn add_edge(&mut self, edge: &E)
    where
        E: Copy,
    {
        // While self loops are not forbidden for contraction, they make it impossible to unpack a shortcut path containing them, as they create a cycle.
        if edge.tail() == edge.head() {
            return;
        }

        self.outgoing.add_edge(*edge);
        self.incoming.add_edge(edge.reversed());
    }

    /// Removes all out and in edges with head == vertex
    pub(super) fn make_unreachable(&mut self, vertex: VertexId) {
        let create_edge = |from| Edge {
            tail: from,
            head: vertex,
        };

        for edge in self.outgoing.out_edges(vertex) {
            self.incoming.remove_edge(create_edge(edge.head())).unwrap();
        }

        for edge in self.incoming.out_edges(vertex) {
            self.outgoing.remove_edge(create_edge(edge.head())).unwrap();
        }
    }

    pub(super) fn edges(self) -> (Vec<E>, Vec<E>)
    where
        E: Send + Sync + Copy,
    {
        rayon::join(
            || self.outgoing.edges().copied().collect(),
            || self.incoming.edges().copied().collect(),
        )
    }
}
