mod cost_of_queries;
mod deleted_neighbors;
mod edge_difference;

use crate::{
    contraction_hierachy::ContractionEdge,
    graph::DirectionalAdjacencyListGraph,
    types::{Distance, VertexId},
};

use cost_of_queries::CostOfQueries;
use deleted_neighbors::DeletedNeighbors;
use edge_difference::EdgeDifference;

pub(super) trait Term<D: Distance>: Send + Sync {
    fn value(
        &self,
        graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
        vertex: VertexId,
        shortcuts: &[ContractionEdge<D>],
    ) -> i64;

    /// Update the state of `Term` as `vertex` is contracted.
    /// This is called before `vertex` is contracted, so neighbors can be reached.
    fn update(
        &mut self,
        graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
        vertex: VertexId,
        shortcuts: &[ContractionEdge<D>],
    );
}

/// Visits each distinct incoming or outgoing neighbor of `vertex`.
fn for_each_neighbor<D: Distance>(
    graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    vertex: VertexId,
    mut visit: impl FnMut(VertexId),
) {
    let out_edges = graph.forward_graph().out_edges(vertex);
    let in_edges = graph.reverse_graph().out_edges(vertex);

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

        visit(neighbor);
    }

    while out_idx < out_edges.len() {
        visit(out_edges[out_idx].head);
        out_idx += 1;
    }

    while in_idx < in_edges.len() {
        visit(in_edges[in_idx].head);
        in_idx += 1;
    }
}

pub(super) fn default_terms<D: Distance>(vertex_count: usize) -> Vec<Box<dyn Term<D>>> {
    vec![
        Box::new(EdgeDifference::new()),
        Box::new(DeletedNeighbors::new(vertex_count)),
        Box::new(CostOfQueries::new(vertex_count)),
    ]
}

pub(super) fn priority<D: Distance>(
    graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    vertex: VertexId,
    shortcuts: &[ContractionEdge<D>],
    terms: &[Box<dyn Term<D>>],
) -> i64 {
    terms
        .iter()
        .map(|term| term.value(graph, vertex, shortcuts))
        .sum()
}
