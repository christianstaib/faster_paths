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
}

impl<E: EdgeLike> GraphLike for Graph<E> {
    type Edge = E;

    fn out_edges(&self, tail: VertexId) -> &[E] {
        if tail.as_usize() >= self.out_edges.len() {
            return &[];
        }

        &self.out_edges[tail.as_usize()]
    }

    fn num_vertices(&self) -> usize {
        self.out_edges
            .iter()
            .flatten()
            .map(|edge| std::cmp::max(edge.tail(), edge.head()))
            .max()
            .map(|vertex| vertex.as_usize())
            .unwrap_or(0)
    }

    fn num_edges(&self) -> usize {
        self.out_edges.iter().map(|edges| edges.len()).sum()
    }
}
