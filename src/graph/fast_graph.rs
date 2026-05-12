use crate::{
    flattened_nested::FlattenedNested, graph::GraphLike, graph::edge_like::EdgeLike,
    types::VertexId,
};
use serde::{Deserialize, Serialize};

/// A graph represented in Compressed Sparse Row (CSR) format.
#[derive(Serialize, Deserialize)]
pub struct FastGraph<E: EdgeLike> {
    flattened_nested: FlattenedNested<E>,
}

impl<E: EdgeLike> FastGraph<E> {
    pub fn new(nested: &Vec<Vec<E>>) -> Self
    where
        E: Copy,
    {
        Self {
            flattened_nested: FlattenedNested::new(nested),
        }
    }

    pub fn from_flat(flat: Vec<E>) -> Self {
        Self {
            flattened_nested: FlattenedNested::from_flat(flat),
        }
    }
}

impl<E: EdgeLike> GraphLike for FastGraph<E> {
    type Edge = E;

    fn out_edges(&self, tail: VertexId) -> &[E] {
        self.flattened_nested.nested(tail.as_usize())
    }

    fn num_vertices(&self) -> usize {
        self.flattened_nested.num_nested()
    }

    fn num_edges(&self) -> usize {
        self.flattened_nested.num_flat()
    }
}
