use crate::types::{Distance, VertexId};

pub struct Edge {
    tail: VertexId,
    target: VertexId,
    weight: Distance,
}

impl Edge {
    pub fn new(tail: VertexId, target: VertexId, weight: Distance) -> Self {
        Self {
            tail,
            target,
            weight,
        }
    }

    pub fn tail(&self) -> VertexId {
        self.tail
    }

    pub fn target(&self) -> VertexId {
        self.target
    }

    pub fn weight(&self) -> Distance {
        self.weight
    }
}
