use crate::{contraction_hierachy::edge::ContractionEdge, graph::FastGraph, types::Distance};

pub struct ContractionHierarchy<D: Distance> {
    up_graph: FastGraph<ContractionEdge<D>>,
    down_graph: FastGraph<ContractionEdge<D>>,
}

impl<D: Distance> ContractionHierarchy<D> {
    pub fn new(
        up_graph: FastGraph<ContractionEdge<D>>,
        down_graph: FastGraph<ContractionEdge<D>>,
    ) -> Self {
        Self {
            up_graph,
            down_graph,
        }
    }

    pub fn up_graph(&self) -> &FastGraph<ContractionEdge<D>> {
        &self.up_graph
    }

    pub fn down_graph(&self) -> &FastGraph<ContractionEdge<D>> {
        &self.down_graph
    }
}
