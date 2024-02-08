use serde_derive::{Deserialize, Serialize};

use super::types::VertexId;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Debug)]
pub struct DirectedWeightedEdge {
    pub head: VertexId,
    pub tail: VertexId,
    pub cost: u32,
}

impl DirectedWeightedEdge {
    pub fn new(tail: VertexId, head: VertexId, cost: u32) -> DirectedWeightedEdge {
        DirectedWeightedEdge { head, tail, cost }
    }

    pub fn inverted(&self) -> DirectedWeightedEdge {
        DirectedWeightedEdge {
            head: self.tail,
            tail: self.head,
            cost: self.cost,
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
            cost: self.cost,
        }
    }

    pub fn headless(&self) -> DirectedHeadlessWeightedEdge {
        DirectedHeadlessWeightedEdge {
            tail: self.tail,
            cost: self.cost,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectedTaillessWeightedEdge {
    pub head: VertexId,
    pub cost: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectedHeadlessWeightedEdge {
    pub tail: VertexId,
    pub cost: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct DirectedEdge {
    pub tail: VertexId,
    pub head: VertexId,
}
