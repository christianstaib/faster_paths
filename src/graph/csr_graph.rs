use crate::{
    flattened_nested::FlattenedNested, graph::GraphLike, graph::edge_like::EdgeLike,
    types::VertexId,
};
use serde::{Deserialize, Serialize};

/// A graph represented in Compressed Sparse Row (CSR) format.
#[derive(Serialize, Deserialize)]
pub struct CsrGraph<E: EdgeLike> {
    flattened_nested: FlattenedNested<E>,
}

impl<E: EdgeLike> CsrGraph<E> {
    pub fn from_flat(mut flat: Vec<E>) -> Self
    where
        E: Copy,
    {
        flat.sort_unstable_by_key(|edge| (edge.tail(), edge.head(), edge.weight()));

        let largest_tail = flat.last().map(|edge| edge.tail().as_usize()).unwrap_or(0);
        let mut indices = vec![0; largest_tail + 2];

        for edge in &flat {
            indices[edge.tail().as_usize() + 1] += 1;
        }

        for i in 1..indices.len() {
            indices[i] += indices[i - 1];
        }

        Self {
            flattened_nested: FlattenedNested::from_flat(flat, indices),
        }
    }

    pub fn new(nested: &Vec<Vec<E>>) -> Self
    where
        E: Copy,
    {
        Self {
            flattened_nested: FlattenedNested::new(nested),
        }
    }
}

impl<E: EdgeLike> GraphLike for CsrGraph<E> {
    type Edge = E;

    fn outgoing_edges(&self, tail: VertexId) -> &[E] {
        self.flattened_nested.nested(tail.as_usize())
    }

    fn num_vertices(&self) -> usize {
        self.flattened_nested.num_nested()
    }

    fn num_edges(&self) -> usize {
        self.flattened_nested.num_flat()
    }
}
