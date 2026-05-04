use crate::types::{Distance, VertexId};

pub struct Edge {
    tail: VertexId,
    head: VertexId,
    weight: Distance,
}

impl Edge {
    pub fn new(tail: VertexId, target: VertexId, weight: Distance) -> Self {
        Self {
            tail,
            head: target,
            weight,
        }
    }

    pub fn tail(&self) -> VertexId {
        self.tail
    }

    pub fn head(&self) -> VertexId {
        self.head
    }

    pub fn weight(&self) -> Distance {
        self.weight
    }
}
