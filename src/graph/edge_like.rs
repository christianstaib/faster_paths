use crate::types::{Distance, VertexId};

pub trait EdgeLike {
    type Distance: Distance;

    fn tail(&self) -> VertexId;
    fn head(&self) -> VertexId;
    fn weight(&self) -> Self::Distance;
}
