use crate::{
    contraction_hierachy::{
        ContractionEdge, contraction::terms::Term, contraction::working_graph::WorkingGraph,
    },
    types::{Distance, VertexId},
};

pub(crate) struct CostOfQueries {
    estimates: Vec<i64>,
}

impl CostOfQueries {
    pub(crate) fn new(vertex_count: usize) -> Self {
        Self {
            estimates: vec![0; vertex_count],
        }
    }

    fn value(&self, vertex: VertexId) -> i64 {
        self.estimates[vertex.as_usize()]
    }
}

impl<D: Distance> Term<D> for CostOfQueries {
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
        let neighbor_estimate = self.value(vertex) + 1;
        let out_edges = graph.get_out(vertex);
        let in_edges = graph.get_in(vertex);

        let mut out_idx = 0;
        let mut in_idx = 0;
        let mut update_estimate = |neighbor: VertexId| {
            let estimate = &mut self.estimates[neighbor.as_usize()];
            *estimate = (*estimate).max(neighbor_estimate);
        };

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

            update_estimate(neighbor);
        }

        while out_idx < out_edges.len() {
            update_estimate(out_edges[out_idx].head);
            out_idx += 1;
        }

        while in_idx < in_edges.len() {
            update_estimate(in_edges[in_idx].head);
            in_idx += 1;
        }
    }
}
