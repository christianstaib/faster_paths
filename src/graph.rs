use crate::{edge_like::EdgeLike, graph_like::GraphLike, types::VertexId};

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

    pub fn out_edges(&self, tail: VertexId) -> &[E] {
        let tail = tail.as_usize();

        if tail > self.out_edges.len() {
            return &[];
        }

        &self.out_edges[tail]
    }

    pub fn num_vertices(&self) -> usize {
        self.out_edges
            .iter()
            .flatten()
            .map(|edge| std::cmp::max(edge.tail(), edge.head()))
            .max()
            .map(|vertex| vertex.as_usize())
            .unwrap_or(0)
    }

    pub fn num_edges(&self) -> usize {
        self.out_edges.iter().map(|edges| edges.len()).sum()
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
