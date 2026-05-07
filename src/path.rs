use crate::types::{Distance, VertexId};

/// Stores a query for a path from source to target.
#[derive(Clone, Copy, Debug)]
pub struct PathQuery {
    pub source: VertexId,
    pub target: VertexId,
}

/// Stores a PathQuery as well as the expected shotests path distance, which is given as an
/// optional, as there might be no path between source and target.
#[derive(Clone, Copy, Debug)]
pub struct PathDistance<D: Distance> {
    query: PathQuery,
    distance: Option<D>,
}

impl<D: Distance> PathDistance<D> {
    pub fn new(query: PathQuery, distance: Option<D>) -> Self {
        Self { query, distance }
    }

    pub fn query(&self) -> &PathQuery {
        &self.query
    }

    pub fn distance(&self) -> Option<D> {
        self.distance
    }
}

/// Stores a path and the distance the paths represents. The first vertex from vertices is the
/// source vertex of the path, the last one the target vertex. As this represents an existing path,
/// vertices shall be non empty.
#[derive(Clone, Debug)]
pub struct Path<D: Distance> {
    vertices: Vec<VertexId>,
    distance: D,
}

impl<D: Distance> Path<D> {
    pub fn new(vertices: Vec<VertexId>, distance: D) -> Self {
        Self { vertices, distance }
    }

    pub fn vertices(&self) -> &[VertexId] {
        &self.vertices
    }

    pub fn distance(&self) -> D {
        self.distance
    }
}
