use crate::{
    Edge as ChEdge,
    edge::Edge,
    flattened_nested::FlattenedNested,
    types::{Distance, VertexId},
};

pub(super) struct WorkingGraph {
    outgoing: Vec<Vec<ChEdge>>,
    incoming: Vec<Vec<(VertexId, Distance)>>,
    contracted_edges: Vec<ChEdge>,
}

impl WorkingGraph {
    /// Given a normal graph, build a `WorkingGraph` which is used during contraction.
    pub(super) fn new(graph: &FlattenedNested<Edge>) -> WorkingGraph {
        let mut working_graph = Self {
            outgoing: vec![Vec::new(); graph.num_nested()],
            incoming: vec![Vec::new(); graph.num_nested()],
            contracted_edges: Vec::new(),
        };

        for bucket in 0..graph.num_nested() {
            for edge in graph.nested(bucket) {
                working_graph.add_edge(ChEdge::new(edge.tail, edge.head, edge.weight, None));
            }
        }

        working_graph
    }

    pub(super) fn num_vertices(&self) -> usize {
        self.outgoing.len()
    }

    pub(super) fn get_out(&self, vertex: VertexId) -> &[ChEdge] {
        return &self.outgoing[vertex.as_usize()];
    }

    pub(super) fn get_in(&self, vertex: VertexId) -> &[(VertexId, Distance)] {
        return &self.incoming[vertex.as_usize()];
    }

    /// Insertes an edge into the graph and sets it as a in-neighbor for its tail.
    pub(super) fn add_edge(&mut self, edge: ChEdge) {
        match self.outgoing[edge.tail.as_usize()]
            .binary_search_by(|old_edge| old_edge.head.cmp(&edge.head))
        {
            Ok(out_idx) => {
                if self.outgoing[edge.tail.as_usize()][out_idx].weight <= edge.weight {
                    return;
                }

                let in_idx = self.incoming[edge.head.as_usize()]
                    .binary_search_by(|(incoming_tail, _)| incoming_tail.cmp(&edge.tail))
                    .expect("incoming edge missing although outgoing edge exists");

                self.incoming[edge.head.as_usize()][in_idx].1 = edge.weight;
                self.outgoing[edge.tail.as_usize()][out_idx] = edge;
            }

            Err(out_idx) => {
                let in_idx = self.incoming[edge.head.as_usize()]
                    .binary_search_by(|(incoming_tail, _)| incoming_tail.cmp(&edge.tail))
                    .expect_err("incoming edge exists although outgoing edge is missing");

                self.incoming[edge.head.as_usize()].insert(in_idx, (edge.tail, edge.weight));
                self.outgoing[edge.tail.as_usize()].insert(out_idx, edge);
            }
        }
    }

    /// Removes all out edges (_ -> v) and all in edges (v <- _), e.g. makes it unreachable from
    /// other vertices.
    pub(super) fn contract_vertex(&mut self, vertex: VertexId) {
        let vertex_index = vertex.as_usize();

        for (tail, _) in std::mem::take(&mut self.incoming[vertex_index]) {
            let edge = {
                let outgoing = &mut self.outgoing[tail.as_usize()];

                let index = outgoing
                    .binary_search_by(|edge| edge.head.cmp(&vertex))
                    .expect("outgoing edge missing although incoming edge exists");

                outgoing.remove(index)
            };

            self.contracted_edges.push(edge);
        }

        for edge in std::mem::take(&mut self.outgoing[vertex_index]) {
            let index = self.incoming[edge.head.as_usize()]
                .binary_search_by(|(tail, _)| tail.cmp(&vertex))
                .expect("incoming edge missing although outgoing edge exists");

            self.incoming[edge.head.as_usize()].remove(index);
            self.contracted_edges.push(edge);
        }
    }

    pub(super) fn contracted_edges(&self) -> &[ChEdge] {
        &self.contracted_edges
    }
}
