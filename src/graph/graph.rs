use crate::{graph::GraphLike, graph::edge_like::EdgeLike, types::VertexId};

/// A graph represented by adjacency lists.
pub struct Graph<E: EdgeLike> {
    out_edges: Vec<Vec<E>>,
}

impl<E: EdgeLike> Graph<E> {
    pub fn new(nested: &Vec<Vec<E>>) -> Self
    where
        E: Copy,
    {
        Self {
            out_edges: nested.clone(),
        }
    }

    pub(crate) fn empty(num_vertices: usize) -> Self {
        let mut out_edges = Vec::with_capacity(num_vertices);
        out_edges.resize_with(num_vertices, Vec::new);

        Self { out_edges }
    }

    pub fn out_edges(&self, tail: VertexId) -> &[E] {
        if tail.as_usize() >= self.out_edges.len() {
            return &[];
        }

        &self.out_edges[tail.as_usize()]
    }

    pub(crate) fn out_edges_mut(&mut self, tail: VertexId) -> &mut Vec<E> {
        &mut self.out_edges[tail.as_usize()]
    }

    pub fn num_vertices(&self) -> usize {
        self.out_edges.len()
    }

    pub fn num_edges(&self) -> usize {
        self.out_edges.iter().map(|edges| edges.len()).sum()
    }

    pub fn edges(&self) -> impl Iterator<Item = &E> + '_ {
        (0..self.num_vertices())
            .map(|tail| VertexId::new(tail as u32))
            .flat_map(|tail| self.out_edges(tail).iter())
    }

    pub(crate) fn into_nested(self) -> Vec<Vec<E>> {
        self.out_edges
    }
}

impl<E: EdgeLike> GraphLike for Graph<E> {
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
