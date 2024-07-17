use serde::{Deserialize, Serialize};

use super::{VertexId, Weight};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Debug)]
pub struct WeightedEdge {
    tail: VertexId,
    head: VertexId,
    weight: Weight,
}

impl WeightedEdge {
    pub fn new(tail: VertexId, head: VertexId, weight: Weight) -> Option<WeightedEdge> {
        if tail == head {
            return None;
        }

        Some(WeightedEdge { head, tail, weight })
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

    pub fn reversed(&self) -> WeightedEdge {
        WeightedEdge {
            head: self.tail,
            tail: self.head,
            weight: self.weight,
        }
    }

    pub fn unweighted(&self) -> Edge {
        Edge {
            tail: self.tail,
            head: self.head,
        }
    }

    pub fn tailless(&self) -> TaillessWeightedEdge {
        TaillessWeightedEdge {
            head: self.head,
            weight: self.weight,
        }
    }

    pub fn headless(&self) -> HeadlessWeightedEdge {
        HeadlessWeightedEdge {
            tail: self.tail,
            weight: self.weight,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaillessWeightedEdge {
    head: VertexId,
    weight: Weight,
}

impl TaillessWeightedEdge {
    pub fn new(head: VertexId, weight: Weight) -> TaillessWeightedEdge {
        TaillessWeightedEdge { head, weight }
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

    pub fn set_tail(&self, tail: VertexId) -> Option<WeightedEdge> {
        WeightedEdge::new(tail, self.head, self.weight)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HeadlessWeightedEdge {
    tail: VertexId,
    weight: Weight,
}

impl HeadlessWeightedEdge {
    pub fn tail(&self) -> VertexId {
        self.tail
    }

    pub fn weight(&self) -> Weight {
        self.weight
    }

    pub fn set_weight(&mut self, weight: Weight) {
        self.weight = weight;
    }

    pub fn set_head(&self, head: VertexId) -> Option<WeightedEdge> {
        WeightedEdge::new(self.tail, head, self.weight)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Edge {
    tail: VertexId,
    head: VertexId,
}

impl Edge {
    pub fn new(tail: VertexId, head: VertexId) -> Option<Edge> {
        if tail == head {
            return None;
        }

        Some(Edge { tail, head })
    }

    pub fn reversed(&self) -> Edge {
        Edge {
            tail: self.head,
            head: self.tail,
        }
    }

    pub fn tail(&self) -> VertexId {
        self.tail
    }

    pub fn head(&self) -> VertexId {
        self.head
    }
}
