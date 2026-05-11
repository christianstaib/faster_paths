mod cost_of_queries;
mod deleted_neighbors;
mod edge_difference;

use crate::{
    contraction_hierachy::ContractionEdge,
    contraction_hierachy::contraction::working_graph::WorkingGraph,
    types::{Distance, VertexId},
};

use cost_of_queries::CostOfQueries;
use deleted_neighbors::DeletedNeighbors;
use edge_difference::EdgeDifference;

pub(super) trait Term<D: Distance>: Send + Sync {
    fn value(
        &self,
        graph: &WorkingGraph<D>,
        vertex: VertexId,
        shortcuts: &[ContractionEdge<D>],
    ) -> i64;

    /// Update the state of `Term` as vertex is contracted.
    /// This is called *before* vertex is constracted, so neighbors can be reached.
    fn update(
        &mut self,
        graph: &WorkingGraph<D>,
        vertex: VertexId,
        shortcuts: &[ContractionEdge<D>],
    );
}

pub(super) fn default_terms<D: Distance>(vertex_count: usize) -> Vec<Box<dyn Term<D>>> {
    vec![
        Box::new(EdgeDifference::new()),
        Box::new(DeletedNeighbors::new(vertex_count)),
        Box::new(CostOfQueries::new(vertex_count)),
    ]
}

pub(super) fn priority<D: Distance>(
    graph: &WorkingGraph<D>,
    vertex: VertexId,
    shortcuts: &[ContractionEdge<D>],
    terms: &[Box<dyn Term<D>>],
) -> i64 {
    terms
        .iter()
        .map(|term| term.value(graph, vertex, shortcuts))
        .sum()
}
