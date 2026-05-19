use crate::{
    contraction_hierachy::ContractionEdge,
    graph::{Edge, EdgeLike, Graph, GraphLike},
    types::{Distance, VertexId},
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

impl<D: Distance> WorkingGraph<ContractionEdge<D>> {
    /// Given a normal graph, build a `WorkingGraph` which is used during contraction.
    pub(super) fn new<G>(graph: &G) -> WorkingGraph<ContractionEdge<D>>
    where
        G: GraphLike,
        G::Edge: EdgeLike<Distance = D>,
    {
        let empty_graph =
            || Graph::from_nested((0..graph.num_vertices()).map(|_| Vec::new()).collect());

        let mut working_graph = Self {
            outgoing: empty_graph(),
            incoming: empty_graph(),
        };

        for edge in graph.edges() {
            working_graph.add_edge(&ContractionEdge {
                tail: edge.tail(),
                head: edge.head(),
                weight: edge.weight(),
                skipped: None,
            });
        }

        working_graph
    }

    /// Inserts an edge into the graph and records its reverse adjacency for the head.
    pub(super) fn add_edge(&mut self, edge: &ContractionEdge<D>) {
        // While self loops are not forbidden for contraction, they make it impossible to unpack a shortcut path containing them, as they create a cycle.
        if edge.tail == edge.head {
            return;
        }

        self.outgoing.add_edge(*edge);
        self.incoming.add_edge(edge.reversed());
    }

    /// Removes all out and in edges with head == vertex
    pub(super) fn make_unreachable(&mut self, vertex: VertexId) {
        {
            let outgoing = &self.outgoing;
            let incoming = &mut self.incoming;

            for edge in outgoing.out_edges(vertex) {
                incoming
                    .remove_edge(Edge {
                        tail: edge.head,
                        head: vertex,
                    })
                    .expect("incoming edge missing although outgoing edge exists");
            }
        }

        {
            let incoming = &self.incoming;
            let outgoing = &mut self.outgoing;

            for incoming_edge in incoming.out_edges(vertex) {
                outgoing
                    .remove_edge(Edge {
                        tail: incoming_edge.head,
                        head: vertex,
                    })
                    .expect("outgoing edge missing although incoming edge exists");
            }
        }
    }

    pub(super) fn edges(self) -> (Vec<ContractionEdge<D>>, Vec<ContractionEdge<D>>)
    where
        D: Send,
    {
        let outgoing = self.outgoing;
        let incoming = self.incoming;

        rayon::join(
            move || outgoing.edges().copied().collect(),
            move || incoming.edges().copied().collect(),
        )
    }
}
