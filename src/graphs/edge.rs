use serde_derive::{Deserialize, Serialize};

use super::{VertexId, Weight};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Debug)]
pub struct DirectedWeightedEdge {
    pub tail: VertexId,
    pub head: VertexId,
    pub weight: Weight,
}

impl DirectedWeightedEdge {
    pub fn new(tail: VertexId, head: VertexId, weight: Weight) -> DirectedWeightedEdge {
        DirectedWeightedEdge { head, tail, weight }
    }

    pub fn inverted(&self) -> DirectedWeightedEdge {
        DirectedWeightedEdge {
            head: self.tail,
            tail: self.head,
            weight: self.weight,
        }
    }

    pub fn unweighted(&self) -> DirectedEdge {
        DirectedEdge {
            tail: self.tail,
            head: self.head,
        }
    }

    pub fn tailless(&self) -> DirectedTaillessWeightedEdge {
        DirectedTaillessWeightedEdge {
            head: self.head,
            weight: self.weight,
        }
    }

    pub fn headless(&self) -> DirectedHeadlessWeightedEdge {
        DirectedHeadlessWeightedEdge {
            tail: self.tail,
            weight: self.weight,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectedTaillessWeightedEdge {
    pub head: VertexId,
    pub weight: Weight,
}

impl DirectedTaillessWeightedEdge {
    pub fn set_tail(&self, tail: VertexId) -> DirectedWeightedEdge {
        DirectedWeightedEdge {
            tail,
            head: self.head,
            weight: self.weight,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectedHeadlessWeightedEdge {
    pub tail: VertexId,
    pub weight: Weight,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct DirectedEdge {
    pub tail: VertexId,
    pub head: VertexId,
}
