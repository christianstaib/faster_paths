use super::{Term, for_each_neighbor};

use crate::{
    contraction_hierachy::ContractionEdge,
    graph::WorkingGraph,
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
        _graph: &WorkingGraph<ContractionEdge<D>>,
        vertex: VertexId,
        _shortcuts: &[ContractionEdge<D>],
    ) -> i64 {
        self.value(vertex)
    }

    fn update(
        &mut self,
        graph: &WorkingGraph<ContractionEdge<D>>,
        vertex: VertexId,
        _shortcuts: &[ContractionEdge<D>],
    ) {
        let neighbor_estimate = self.value(vertex) + 1;
        for_each_neighbor(graph, vertex, |neighbor| {
            let estimate = &mut self.estimates[neighbor.as_usize()];
            *estimate = (*estimate).max(neighbor_estimate);
        });
    }
}
