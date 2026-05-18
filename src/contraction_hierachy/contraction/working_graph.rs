use rayon::slice::ParallelSliceMut;

use crate::{
    contraction_hierachy::ContractionEdge,
    graph::{EdgeLike, GraphLike},
    types::{Distance, VertexId},
};

pub(super) struct WorkingGraph<D: Distance> {
    outgoing: Vec<Vec<ContractionEdge<D>>>,
    incoming: Vec<Vec<ContractionEdge<D>>>,
}

impl<D: Distance> WorkingGraph<D> {
    /// Given a normal graph, build a `WorkingGraph` which is used during contraction.
    pub(super) fn new<G>(graph: &G) -> WorkingGraph<D>
    where
        G: GraphLike,
        G::Edge: EdgeLike<Distance = D>,
    {
        let mut working_graph = Self {
            outgoing: vec![Vec::new(); graph.num_vertices()],
            incoming: vec![Vec::new(); graph.num_vertices()],
        };

        for edge in graph.edges() {
            working_graph.add_edge(ContractionEdge {
                tail: edge.tail(),
                head: edge.head(),
                weight: edge.weight(),
                skipped: None,
            });
        }

        working_graph
    }

    pub(super) fn num_vertices(&self) -> usize {
        self.outgoing.len()
    }

    pub(super) fn get_out(&self, vertex: VertexId) -> &[ContractionEdge<D>] {
        return &self.outgoing[vertex.as_usize()];
    }

    /// Returns reverse incoming adjacency, stored as `vertex -> predecessor`.
    pub(super) fn get_in(&self, vertex: VertexId) -> &[ContractionEdge<D>] {
        return &self.incoming[vertex.as_usize()];
    }

    /// Inserts an edge into the graph and records its reverse adjacency for the head.
    pub(super) fn add_edge(&mut self, edge: ContractionEdge<D>) {
        // While self loops are not forbidden for contraction, they make it impossible to unpack a shortcut path containing them, as they create a cycle.
        if edge.tail == edge.head {
            return;
        }

        match self.outgoing[edge.tail.as_usize()]
            .binary_search_by(|old_edge| old_edge.head.cmp(&edge.head))
        {
            Ok(out_idx) => {
                // Edge already in graph. Update weight.
                if self.outgoing[edge.tail.as_usize()][out_idx].weight <= edge.weight {
                    return;
                }

                let in_idx = self.incoming[edge.head.as_usize()]
                    .binary_search_by(|incoming_edge| incoming_edge.head.cmp(&edge.tail))
                    .expect("incoming edge missing although outgoing edge exists");

                self.incoming[edge.head.as_usize()][in_idx].weight = edge.weight;
                self.outgoing[edge.tail.as_usize()][out_idx] = edge;
            }

            Err(out_idx) => {
                let in_idx = self.incoming[edge.head.as_usize()]
                    .binary_search_by(|incoming_edge| incoming_edge.head.cmp(&edge.tail))
                    .expect_err("incoming edge already exists although outgoing edge is missing");

                self.incoming[edge.head.as_usize()].insert(in_idx, edge.reversed());
                self.outgoing[edge.tail.as_usize()].insert(out_idx, edge);
            }
        }
    }

    /// Removes all active incident edges of `vertex`.
    pub(super) fn contract_vertex(&mut self, vertex: VertexId) {
        let vertex_index = vertex.as_usize();

        for edge in &self.outgoing[vertex_index] {
            let index = self.incoming[edge.head.as_usize()]
                .binary_search_by(|incoming_edge| incoming_edge.head.cmp(&vertex))
                .expect("incoming edge missing although outgoing edge exists");

            self.incoming[edge.head.as_usize()].remove(index);
        }

        for incoming_edge in &self.incoming[vertex_index] {
            let outgoing = &mut self.outgoing[incoming_edge.head.as_usize()];

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
        let num_up_edges: usize = self.outgoing.iter().map(|edges| edges.len()).sum();
        let num_down_edges: usize = self.incoming.iter().map(|edges| edges.len()).sum();

        let mut up_edges = Vec::with_capacity(num_up_edges);
        let mut down_edges = Vec::with_capacity(num_down_edges);

        for mut edges in self.outgoing {
            up_edges.extend_from_slice(&edges);
            edges.clear();
        }

        for mut edges in self.incoming {
            down_edges.extend_from_slice(&edges);
            edges.clear();
        }

        up_edges.par_sort_unstable();
        down_edges.par_sort_unstable();

        (up_edges, down_edges)
    }
}
