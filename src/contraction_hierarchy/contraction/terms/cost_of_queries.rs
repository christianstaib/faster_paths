use super::{Term, for_each_neighbor};

use crate::{
    contraction_hierarchy::ContractionEdge,
    graph::DirectionalAdjacencyListGraph,
    types::{Distance, Vertex},
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

    fn value(&self, vertex: Vertex) -> i64 {
        self.estimates[vertex as usize]
    }
}

impl<D: Distance> Term<D> for CostOfQueries {
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
        let neighbor_estimate = self.value(vertex) + 1;
        for_each_neighbor(graph, vertex, |neighbor| {
            let estimate = &mut self.estimates[neighbor as usize];
            *estimate = (*estimate).max(neighbor_estimate);
        });
    }
}
