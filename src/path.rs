use crate::types::{Distance, Vertex};
use rand::seq::index::sample;
use serde::{Deserialize, Serialize};
use std::iter;

/// Query for a path from a source vertex to a target vertex.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Query {
    /// Source vertex of the query.
    pub source: Vertex,

    /// Target vertex of the query.
    pub target: Vertex,
}

/// A concrete path.
///
/// Since `Path` represents an existing path, `vertices` should be non-empty.
#[derive(Clone, Debug)]
pub struct Path<D: Distance> {
    /// Ordered vertices of the path: `vertices.first()` is the source, and
    /// `vertices.last()` is the target.
    pub vertices: Vec<Vertex>,

    /// Total cost of the path.
    ///
    /// This should equal the sum of the weights of all consecutive edges in
    /// `vertices`.
    pub distance: D,
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
