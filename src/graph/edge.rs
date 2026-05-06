use crate::{
    graph::edge_like::EdgeLike,
    types::{Distance, VertexId},
};

#[derive(Clone, Copy)]
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

impl EdgeLike for Edge {
    fn tail(&self) -> VertexId {
        self.tail
    }
    fn head(&self) -> VertexId {
        self.head
    }
    fn weight(&self) -> Distance {
        self.weight
    }
}
