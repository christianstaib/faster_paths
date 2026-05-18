use crate::{
    graph::EdgeLike,
    types::{Distance, VertexId},
};

/// A ContractionEdge is stored either in the `up_graph` or `down_graph` of a
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContractionEdge<D> {
    pub tail: VertexId,
    pub head: VertexId,
    pub weight: D,
    pub skipped: Option<VertexId>,
}

impl<D: Distance> ContractionEdge<D> {
    pub fn reversed(&self) -> Self {
        Self {
            tail: self.head,
            head: self.tail,
            ..*self
        }
    }
}

impl<D: Distance> EdgeLike for ContractionEdge<D> {
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
}
