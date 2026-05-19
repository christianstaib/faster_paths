use crate::{
    contraction_hierachy::ContractionEdge,
    graph::{EdgeLike, Graph, GraphLike},
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
        let mut working_graph = Self {
            outgoing: Graph::empty(graph.num_vertices()),
            incoming: Graph::empty(graph.num_vertices()),
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

        match self
            .outgoing
            .out_edges(edge.tail)
            .binary_search_by(|old_edge| old_edge.head.cmp(&edge.head))
        {
            Ok(out_idx) => {
                // Edge already in graph. Update weight.
                if self.outgoing.out_edges(edge.tail)[out_idx].weight <= edge.weight {
                    return;
                }

                let in_idx = self
                    .incoming
                    .out_edges(edge.head)
                    .binary_search_by(|incoming_edge| incoming_edge.head.cmp(&edge.tail))
                    .expect("incoming edge missing although outgoing edge exists");

                self.incoming.out_edges_mut(edge.head)[in_idx].weight = edge.weight;
                self.outgoing.out_edges_mut(edge.tail)[out_idx].weight = edge.weight;
            }

            Err(out_idx) => {
                let in_idx = self
                    .incoming
                    .out_edges(edge.head)
                    .binary_search_by(|incoming_edge| incoming_edge.head.cmp(&edge.tail))
                    .expect_err("incoming edge already exists although outgoing edge is missing");

                self.incoming
                    .out_edges_mut(edge.head)
                    .insert(in_idx, edge.reversed());
                self.outgoing
                    .out_edges_mut(edge.tail)
                    .insert(out_idx, edge.clone());
            }
        }
    }

    /// Removes all out and in edges with head == vertex
    pub(super) fn make_unreachable(&mut self, vertex: VertexId) {
        for edge in self.outgoing.out_edges(vertex) {
            let index = self
                .incoming
                .out_edges(edge.head)
                .binary_search_by(|incoming_edge| incoming_edge.head.cmp(&vertex))
                .expect("incoming edge missing although outgoing edge exists");

            self.incoming.out_edges_mut(edge.head).remove(index);
        }

        for incoming_edge in self.incoming.out_edges(vertex) {
            let outgoing = self.outgoing.out_edges_mut(incoming_edge.head);

            let index = outgoing
                .binary_search_by(|edge| edge.head.cmp(&vertex))
                .expect("outgoing edge missing although incoming edge exists");

            outgoing.remove(index);
        }
    }

    pub(super) fn edges(self) -> (Vec<ContractionEdge<D>>, Vec<ContractionEdge<D>>)
    where
        D: Send,
    {
        let flatten = |graph: Graph<ContractionEdge<D>>| {
            let nested = graph.into_nested();
            let mut flat = Vec::with_capacity(nested.iter().map(Vec::len).sum());

            for mut chunk in nested {
                flat.append(&mut chunk);
            }

            flat
        };

        let outgoing = self.outgoing;
        let incoming = self.incoming;

        rayon::join(move || flatten(outgoing), move || flatten(incoming))
    }
}
