use crate::{
    graph::edge_like::EdgeLike,
    types::{Distance, VertexId},
};

#[derive(Clone, Copy)]
pub struct Edge {
    pub tail: VertexId,
    pub head: VertexId,
}

#[derive(Clone, Copy, Debug)]
pub struct WeightedEdge<D> {
    pub tail: VertexId,
    pub head: VertexId,
    pub weight: D,
}

impl<D: Distance> EdgeLike for WeightedEdge<D> {
    type Distance = D;

    fn tail(&self) -> VertexId {
        self.tail
    }
    fn head(&self) -> VertexId {
        self.head
    }
    fn weight(&self) -> Self::Distance {
        self.weight
    }

    fn reversed(&self) -> Self {
        Self {
            tail: self.head,
            head: self.tail,
            weight: self.weight,
        }
    }
}
