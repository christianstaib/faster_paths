use crate::{
    graph::EdgeLike,
    types::{Distance, Vertex},
};
use serde::{Deserialize, Serialize};

/// A `ContractionEdge` is stored either in the `up_graph` or `down_graph` of a
/// `ContractionHierarchy`.
///
/// In both graphs, the stored edge is directed upward, i.e.
/// `level(head) > level(tail)`.
///
/// If the edge is *not* a shortcut:
/// - `skipped` is `None`.
///
/// If the edge *is* a shortcut:
/// - `skipped` is `Some(middle)`, where `middle` is the contracted vertex that
///   was skipped by this shortcut.
/// - the child edge in the *other* graph is the edge `middle -> tail`.
/// - the child edge in the *same* graph is the edge `middle -> head`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractionEdge<D> {
    pub tail: Vertex,
    pub head: Vertex,
    pub weight: D,
    pub skipped: Option<Vertex>,
}

impl<D: Distance> EdgeLike for ContractionEdge<D> {
    type Weight = D;

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
            ..*self
        }
    }
}
