use super::{Term, for_each_neighbor};

use crate::{
    contraction_hierarchy::ContractionEdge,
    graph::DirectionalAdjacencyListGraph,
    types::{Distance, Vertex},
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

    fn value(&self, vertex: Vertex) -> i64 {
        self.counts[vertex as usize]
    }
}

impl<D: Distance> Term<D> for DeletedNeighbors {
    fn value(
        &self,
        _graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
        vertex: Vertex,
        _shortcuts: &[ContractionEdge<D>],
    ) -> i64 {
        self.value(vertex)
    }

    fn update(
        &mut self,
        graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
        vertex: Vertex,
        _shortcuts: &[ContractionEdge<D>],
    ) {
        for_each_neighbor(graph, vertex, |neighbor| {
            self.counts[neighbor as usize] += 1;
        });
    }
}
