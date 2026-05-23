use crate::types::{Distance, Vertex};
use rand::seq::index::sample;
use serde::{Deserialize, Serialize};
use std::iter;

/// Stores a query for a path from source to target.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Query {
    pub source: Vertex,
    pub target: Vertex,
}

pub fn generate_random_queries(num_vertices: usize, num_queries: usize) -> Vec<Query> {
    let mut rng = rand::rng();
    iter::repeat_with(|| {
        let [source, target] = sample(&mut rng, num_vertices, 2)
            .into_vec()
            .try_into()
            .unwrap();

        Query {
            source: Vertex::from(source as u32),
            target: Vertex::from(target as u32),
        }
    })
    .take(num_queries)
    .collect()
}

/// Stores a path and the distance the path represents. The first vertex from vertices is the
/// source vertex of the path, the last one the target vertex. As this represents an existing path,
/// vertices should be non-empty.
#[derive(Clone, Debug)]
pub struct Path<D: Distance> {
    pub vertices: Vec<Vertex>,
    pub distance: D,
}
