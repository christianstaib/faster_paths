use crate::{
    edge_like::EdgeLike, flattened_nested::FlattenedNested, graph::GraphLike, types::VertexId,
};

/// A graph represented in Compressed Sparse Row (CSR) format.
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
