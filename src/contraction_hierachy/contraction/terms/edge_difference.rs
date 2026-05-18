use crate::{
    contraction_hierachy::{
        ContractionEdge, contraction::terms::Term, contraction::working_graph::WorkingGraph,
    },
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
        graph: &WorkingGraph<D>,
        vertex: VertexId,
        shortcuts: &[ContractionEdge<D>],
    ) -> i64 {
        let degree = graph.get_out(vertex).len() + graph.get_in(vertex).len();
        shortcuts.len() as i64 - degree as i64
    }

    fn update(
        &mut self,
        _graph: &WorkingGraph<D>,
        _vertex: VertexId,
        _shortcuts: &[ContractionEdge<D>],
    ) {
    }
}
