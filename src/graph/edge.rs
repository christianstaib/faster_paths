use serde::{Deserialize, Serialize};

use crate::{
    graph::edge_like::EdgeLike,
    types::{Distance, VertexId},
};

/// A directed edge.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub tail: VertexId,
    pub head: VertexId,
}

/// A directed edge with an associated weight.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeightedEdge<W> {
    pub tail: VertexId,
    pub head: VertexId,
    pub weight: W,
}

impl<W: Distance> EdgeLike for WeightedEdge<W> {
    type Weight = W;

    fn tail(&self) -> VertexId {
        self.tail
    }
    fn head(&self) -> VertexId {
        self.head
    }
    fn weight(&self) -> Self::Weight {
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
