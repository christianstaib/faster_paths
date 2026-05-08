use crate::{
    contraction_hierachy::ContractionEdge,
    graph::{EdgeLike, GraphLike, WeightedEdge},
    types::{Distance, VertexId},
};

pub(super) struct WorkingGraph<D: Distance> {
    outgoing: Vec<Vec<ContractionEdge<D>>>,
    incoming: Vec<Vec<WeightedEdge<D>>>,
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

    pub(super) fn get_in(&self, vertex: VertexId) -> &[WeightedEdge<D>] {
        return &self.incoming[vertex.as_usize()];
    }

    /// Inserts an edge into the graph and sets it as an in-neighbor for its head.
    pub(super) fn add_edge(&mut self, edge: ContractionEdge<D>) {
        match self.outgoing[edge.tail.as_usize()]
            .binary_search_by(|old_edge| old_edge.head.cmp(&edge.head))
        {
            Ok(out_idx) => {
                // Edge already in graph. Update weight.
                if self.outgoing[edge.tail.as_usize()][out_idx].weight <= edge.weight {
                    return;
                }

                let in_idx = self.incoming[edge.head.as_usize()]
                    .binary_search_by(|incoming_edge| incoming_edge.tail.cmp(&edge.tail))
                    .expect("incoming edge missing although outgoing edge exists");

                self.incoming[edge.head.as_usize()][in_idx].weight = edge.weight;
                self.outgoing[edge.tail.as_usize()][out_idx] = edge;
            }

            Err(out_idx) => {
                let in_idx = self.incoming[edge.head.as_usize()]
                    .binary_search_by(|incoming_edge| incoming_edge.tail.cmp(&edge.tail))
                    .expect_err("incoming edge already exists although outgoing edge is missing");

                self.incoming[edge.head.as_usize()].insert(
                    in_idx,
                    WeightedEdge {
                        tail: edge.tail,
                        head: edge.head,
                        weight: edge.weight,
                    },
                );
                self.outgoing[edge.tail.as_usize()].insert(out_idx, edge);
            }
        }
    }

    /// Removes all out edges (_ -> v) and all in edges (v <- _), e.g. makes it unreachable from
    /// other vertices.
    pub(super) fn contract_vertex(&mut self, vertex: VertexId) {
        let vertex_index = vertex.as_usize();

        for edge in &self.outgoing[vertex_index] {
            let index = self.incoming[edge.head.as_usize()]
                .binary_search_by(|incoming_edge| incoming_edge.tail.cmp(&vertex))
                .expect("incoming edge missing although outgoing edge exists");

            self.incoming[edge.head.as_usize()].remove(index);
        }

        let mut incoming_edges = Vec::new();
        for incoming_edge in std::mem::take(&mut self.incoming[vertex_index]) {
            let edge = {
                let outgoing = &mut self.outgoing[incoming_edge.tail.as_usize()];

                let index = outgoing
                    .binary_search_by(|edge| edge.head.cmp(&vertex))
                    .expect("outgoing edge missing although incoming edge exists");

                outgoing.remove(index)
            };

            incoming_edges.push(edge);
        }

        self.outgoing[vertex_index].extend(incoming_edges);
        self.outgoing[vertex_index].shrink_to_fit();
    }

    pub(super) fn get_edges(&self) -> Vec<ContractionEdge<D>> {
        self.outgoing.iter().cloned().flatten().collect()
    }
}
