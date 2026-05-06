use crate::types::{Distance, VertexId};

pub trait EdgeLike {
    fn tail(&self) -> VertexId;
    fn head(&self) -> VertexId;
    fn weight(&self) -> Distance;
}
