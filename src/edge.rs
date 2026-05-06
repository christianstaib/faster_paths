use crate::types::{Distance, VertexId};

pub struct Edge {
    pub tail: VertexId,
    pub head: VertexId,
    pub weight: Distance,
}

impl Edge {
    pub fn new(tail: VertexId, target: VertexId, weight: Distance) -> Self {
        Self {
            tail,
            head: target,
            weight,
        }
    }
}
