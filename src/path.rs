use crate::types::{Distance, VertexId};

/// Stores a query for a path from source to target.
#[derive(Clone, Copy, Debug)]
pub struct PathQuery {
    source: VertexId,
    target: VertexId,
}

impl PathQuery {
    pub fn new(source: VertexId, target: VertexId) -> Self {
        Self { source, target }
    }

    pub fn source(&self) -> VertexId {
        self.source
    }

    pub fn target(&self) -> VertexId {
        self.target
    }
}

/// Stores a PathQuery as well as the expected shotests path distance, which is given as an
/// optional, as there might be no path between source and target.
#[derive(Clone, Copy, Debug)]
pub struct PathDistance {
    query: PathQuery,
    distance: Option<Distance>,
}

impl PathDistance {
    pub fn new(query: PathQuery, distance: Option<Distance>) -> Self {
        Self { query, distance }
    }

    pub fn query(&self) -> &PathQuery {
        &self.query
    }

    pub fn distance(&self) -> Option<Distance> {
        self.distance
    }
}

/// Stores a path and the distance the paths represents. The first vertex from vertices is the
/// source vertex of the path, the last one the target vertex. As this represents an existing path,
/// vertices shall be non empty.
#[derive(Clone, Debug)]
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
