use serde::{Deserialize, Serialize};

use crate::{
    graph::edge_like::EdgeLike,
    types::{Distance, Vertex},
};

/// A directed edge.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub tail: Vertex,
    pub head: Vertex,
}

/// A directed edge with an associated weight.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeightedEdge<Weight> {
    pub tail: Vertex,
    pub head: Vertex,
    pub weight: Weight,
}

impl<Weight: Distance> EdgeLike for WeightedEdge<Weight> {
    type Weight = Weight;

    fn tail(&self) -> Vertex {
        self.tail
    }
    fn head(&self) -> Vertex {
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
