use crate::types::{Distance, VertexId};

pub struct Path {
    vertices: Vec<VertexId>,
    distance: Distance,
}

impl Path {
    pub fn new(vertices: Vec<VertexId>, distance: Distance) -> Self {
        Self { vertices, distance }
    }

    pub fn vertices(&self) -> &[VertexId] {
        &self.vertices
    }

    pub fn distance(&self) -> Distance {
        self.distance
    }
}
