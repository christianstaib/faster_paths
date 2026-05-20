use crate::{
    contraction_hierachy::{ContractionEdge, contraction::terms::Term},
    graph::DirectionalAdjacencyListGraph,
    types::{Distance, VertexId},
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
        vertex: VertexId,
        shortcuts: &[ContractionEdge<D>],
    ) -> i64 {
        let degree = graph.forward_graph().out_edges(vertex).len()
            + graph.reverse_graph().out_edges(vertex).len();
        shortcuts.len() as i64 - degree as i64
    }

    fn update(
        &mut self,
        _graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
        _vertex: VertexId,
        _shortcuts: &[ContractionEdge<D>],
    ) {
    }
}
