use crate::{
    edge_like::EdgeLike, flattened_nested::FlattenedNested, graph_like::GraphLike, types::VertexId,
};

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

    pub fn out_edges(&self, tail: VertexId) -> &[E] {
        self.flattened_nested.nested(tail.as_usize())
    }

    pub fn num_vertices(&self) -> usize {
        self.flattened_nested.num_nested()
    }

    pub fn num_edges(&self) -> usize {
        self.flattened_nested.num_flat()
    }
}

impl<E: EdgeLike> GraphLike for FastGraph<E> {
    type Edge = E;

    fn out_edges(&self, tail: VertexId) -> &[E] {
        self.out_edges(tail)
    }

    fn num_vertices(&self) -> usize {
        self.num_vertices()
    }

    fn num_edges(&self) -> usize {
        self.num_edges()
    }
}
