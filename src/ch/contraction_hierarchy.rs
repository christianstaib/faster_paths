use crate::{ch::edge::Edge, flattened_nested::FlattenedNested};

pub struct ContractionHierarchy {
    up_graph: FlattenedNested<Edge>,
    down_graph: FlattenedNested<Edge>,
}

impl ContractionHierarchy {
    pub fn new(up_graph: FlattenedNested<Edge>, down_graph: FlattenedNested<Edge>) -> Self {
        Self {
            up_graph,
            down_graph,
        }
    }

    pub fn up_graph(&self) -> &FlattenedNested<Edge> {
        &self.up_graph
    }

    pub fn down_graph(&self) -> &FlattenedNested<Edge> {
        &self.down_graph
    }
}
