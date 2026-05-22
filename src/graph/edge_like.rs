use crate::types::{Distance, Vertex};

/// Common interface for a directed, weighted edge.
///
/// The edge is directed from its tail vertex to its head vertex and has an
/// associated weight of type [`Distance`].
pub trait EdgeLike {
    type Weight: Distance;

    /// Returns the tail vertex of the edge.
    fn tail(&self) -> Vertex;

    /// Returns the head vertex of the edge.
    fn head(&self) -> Vertex;

    /// Returns the weight of the edge.
    fn weight(&self) -> Self::Weight;

    /// Returns the same edge with tail and head swapped.
    fn reversed(&self) -> Self;
}
