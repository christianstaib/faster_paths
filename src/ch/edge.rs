use crate::{
    edge_like::EdgeLike,
    types::{Distance, VertexId},
};

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
pub struct ContractionEdge {
    pub tail: VertexId,
    pub head: VertexId,
    pub weight: Distance,
    pub skipped: Option<VertexId>,
}

impl ContractionEdge {
    pub fn new(
        tail: VertexId,
        head: VertexId,
        weight: Distance,
        skipped: Option<VertexId>,
    ) -> Self {
        Self {
            tail,
            head,
            weight,
            skipped,
        }
    }

    pub fn reversed(&self) -> Self {
        ContractionEdge::new(self.head, self.tail, self.weight, self.skipped)
    }
}

impl EdgeLike for ContractionEdge {
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
