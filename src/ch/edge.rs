use crate::types::{Distance, VertexId};

/// An edge is stored either in the `up_graph` or `down_graph` of a
/// `ContractionHierarchy`.
///
/// In both graphs the stored edge is directed upward, i.e.
/// `level(head) > level(tail)`.
///
/// If the edge is *not* a shortcut:
/// - `skipped` is `None`.
///
/// If the edge *is* a shortcut:
/// - `skipped` is `Some(middle)`, where `middle` is the contracted vertex that
///   was skipped by this shortcut.
/// - the child edge in the *other* graph is an edge `middle -> tail`.
/// - the child edge in the *same* graph is an edge `middle -> head`.
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

    pub fn set_weight(&mut self, weight: Distance) {
        self.weight = weight;
    }

    pub fn set_skiped(&mut self, skiped: Option<VertexId>) {
        self.skiped = skiped;
    }

    pub fn set_tail(&mut self, tail: VertexId) {
        self.tail = tail;
    }

    pub fn set_head(&mut self, head: VertexId) {
        self.head = head;
    }
}
