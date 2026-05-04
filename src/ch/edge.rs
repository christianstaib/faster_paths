use crate::types::{Distance, VertexId};

/// An edge is stored either in the up_graph or down_graph of Data.
/// In both cases it is directed upwards, i.e. level(head_) > level(tail_).
///
/// If the edge is *not* a shortcut:
/// - child1_ is set to the sentinel INVALID_INDEX.
///
/// If the edge *is* a shortcut:
/// - child1_ is the index of the edge in the *other* direction (middle -> tail_).
/// - child2_ is the index of the edge in the *same* direction (middle -> head_).
///
/// tail <------------- (skiped) -------------> head
///       child_edge_1            child_edge_2

#[derive(Clone, Copy)]
pub struct Edge {
    tail: VertexId,
    head: VertexId,
    weight: Distance,
    skiped: Option<VertexId>,
}

impl Edge {
    pub fn new(tail: VertexId, head: VertexId, weight: Distance, skiped: Option<VertexId>) -> Self {
        Self {
            tail,
            head,
            weight,
            skiped,
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

    pub fn skipped(&self) -> Option<VertexId> {
        self.skiped
    }
}
