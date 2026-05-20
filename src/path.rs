use crate::types::{Distance, VertexId};
use rand::seq::index::sample;
use serde::{Deserialize, Serialize};
use std::iter;

/// Stores a query for a path from source to target.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PathQuery {
    pub source: VertexId,
    pub target: VertexId,
}

pub fn generate_queries(num_vertices: usize, num_tests: usize) -> Vec<PathQuery> {
    let mut rng = rand::rng();
    iter::repeat_with(|| {
        let [source, target] = sample(&mut rng, num_vertices, 2)
            .into_vec()
            .try_into()
            .unwrap();

        PathQuery {
            source: VertexId::new(source as u32),
            target: VertexId::new(target as u32),
        }
    })
    .take(num_tests)
    .collect()
}

/// Stores a PathQuery as well as the expected shotests path distance, which is given as an
/// optional, as there might be no path between source and target.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
    pub vertices: Vec<VertexId>,
    pub distance: D,
}
