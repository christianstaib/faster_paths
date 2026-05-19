use super::{Term, for_each_neighbor};

use crate::{
    contraction_hierachy::{ContractionEdge, contraction::working_graph::WorkingGraph},
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
        for_each_neighbor(graph, vertex, |neighbor| {
            self.counts[neighbor.as_usize()] += 1;
        });
    }
}
