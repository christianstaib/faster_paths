use crate::{
    contraction_hierachy::{
        ContractionEdge, contraction::terms::Term, contraction::working_graph::WorkingGraph,
    },
    types::{Distance, VertexId},
};

pub(crate) struct DeletedNeighbors {
    counts: Vec<i64>,
}

impl DeletedNeighbors {
    pub(crate) fn new(vertex_count: usize) -> Self {
        Self {
            counts: vec![0; vertex_count],
        }
    }

    fn value(&self, vertex: VertexId) -> i64 {
        self.counts[vertex.as_usize()]
    }
}

impl<D: Distance> Term<D> for DeletedNeighbors {
    fn value(
        &self,
        _graph: &WorkingGraph<D>,
        vertex: VertexId,
        _shortcuts: &[ContractionEdge<D>],
    ) -> i64 {
        self.value(vertex)
    }

    fn update(
        &mut self,
        graph: &WorkingGraph<D>,
        vertex: VertexId,
        _shortcuts: &[ContractionEdge<D>],
    ) {
        let out_edges = graph.get_out(vertex);
        let in_edges = graph.get_in(vertex);

        let mut out_idx = 0;
        let mut in_idx = 0;

        while out_idx < out_edges.len() && in_idx < in_edges.len() {
            let out_head = out_edges[out_idx].head;
            let in_head = in_edges[in_idx].head;

            let neighbor = match out_head.cmp(&in_head) {
                std::cmp::Ordering::Less => {
                    out_idx += 1;
                    out_head
                }
                std::cmp::Ordering::Greater => {
                    in_idx += 1;
                    in_head
                }
                std::cmp::Ordering::Equal => {
                    out_idx += 1;
                    in_idx += 1;
                    out_head
                }
            };

            self.counts[neighbor.as_usize()] += 1;
        }

        while out_idx < out_edges.len() {
            let neighbor = out_edges[out_idx].head;
            self.counts[neighbor.as_usize()] += 1;
            out_idx += 1;
        }

        while in_idx < in_edges.len() {
            let neighbor = in_edges[in_idx].head;
            self.counts[neighbor.as_usize()] += 1;
            in_idx += 1;
        }
    }
}
