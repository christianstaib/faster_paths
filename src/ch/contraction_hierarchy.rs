use crate::{ch::edge::ContractionEdge, graph::FastGraph};

pub struct ContractionHierarchy {
    up_graph: FastGraph<ContractionEdge>,
    down_graph: FastGraph<ContractionEdge>,
}

impl ContractionHierarchy {
    pub fn new(
        up_graph: FastGraph<ContractionEdge>,
        down_graph: FastGraph<ContractionEdge>,
    ) -> Self {
        Self {
            up_graph,
            down_graph,
        }
    }

    pub fn up_graph(&self) -> &FastGraph<ContractionEdge> {
        &self.up_graph
    }

    pub fn down_graph(&self) -> &FastGraph<ContractionEdge> {
        &self.down_graph
    }
}
