use crate::{
    contraction_hierarchy::{ContractionEdge, contraction::terms::Term},
    graph::{DirectionalAdjacencyListGraph, GraphLike},
    types::{Distance, Vertex},
};

pub(crate) struct EdgeDifference;

impl EdgeDifference {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl<D: Distance> Term<D> for EdgeDifference {
    fn value(
        &self,
        graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
        vertex: Vertex,
        shortcuts: &[ContractionEdge<D>],
    ) -> i64 {
        let degree = graph.forward().outgoing_edges(vertex).len()
            + graph.reverse().outgoing_edges(vertex).len();
        shortcuts.len() as i64 - degree as i64
    }

    fn update(
        &mut self,
        _graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
        _vertex: Vertex,
        _shortcuts: &[ContractionEdge<D>],
    ) {
    }
}
