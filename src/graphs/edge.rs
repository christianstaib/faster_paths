use serde::{Deserialize, Serialize};

use super::{VertexId, Weight};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Debug)]
pub struct DirectedWeightedEdge {
    tail: VertexId,
    head: VertexId,
    weight: Weight,
}

impl DirectedWeightedEdge {
    pub fn new(tail: VertexId, head: VertexId, weight: Weight) -> Option<DirectedWeightedEdge> {
        if tail == head {
            return None;
        }

        Some(DirectedWeightedEdge { head, tail, weight })
    }

    pub fn tail(&self) -> VertexId {
        self.tail
    }

    pub fn head(&self) -> VertexId {
        self.head
    }

    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn reversed(&self) -> DirectedWeightedEdge {
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
    head: VertexId,
    weight: Weight,
}

impl DirectedTaillessWeightedEdge {
    pub fn new(head: VertexId, weight: Weight) -> DirectedTaillessWeightedEdge {
        DirectedTaillessWeightedEdge{
            head, weight
        }
    }

    pub fn head(&self) -> VertexId {
        self.head
    }

    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn set_weight(&mut self, weight: Weight) {
        self.weight = weight;
    }

    pub fn set_tail(&self, tail: VertexId) -> Option<DirectedWeightedEdge> {
        DirectedWeightedEdge::new(tail, self.head, self.weight)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DirectedHeadlessWeightedEdge {
    tail: VertexId,
    weight: Weight,
}

impl DirectedHeadlessWeightedEdge {
    pub fn tail(&self) -> VertexId {
        self.tail
    }

    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn set_weight(&mut self, weight: Weight) {
        self.weight = weight;
    }

    pub fn set_head(&self, head: VertexId) -> Option<DirectedWeightedEdge> {
        DirectedWeightedEdge::new(self.tail, head, self.weight)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct DirectedEdge {
    tail: VertexId,
    head: VertexId,
}

impl DirectedEdge {
    pub fn new(tail: VertexId, head: VertexId) -> Option<DirectedEdge> {
        if tail == head {
            return None;
        }

        Some(DirectedEdge { tail, head })
    }

    pub fn tail(&self) -> VertexId {
        self.tail
    }

    pub fn head(&self) -> VertexId {
        self.head
    }
}
