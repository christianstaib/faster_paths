use serde::{Deserialize, Serialize};

use crate::{
    graph::{Edge, GraphLike, edge_like::EdgeLike},
    types::VertexId,
};

/// A graph represented by adjacency lists.
#[derive(Serialize, Deserialize)]
pub struct AdjacencyListGraph<E: EdgeLike> {
    edges: Vec<Vec<E>>,
}

impl<E: EdgeLike> AdjacencyListGraph<E> {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub fn from_nested(edges: Vec<Vec<E>>) -> Self {
        Self { edges }
    }

    pub fn get_edges(&self) -> &Vec<Vec<E>> {
        &self.edges
    }

    pub fn add_edge(&mut self, edge: &E)
    where
        E: Copy,
    {
        if edge.tail() == edge.head() {
            return;
        }

        let needed_len = std::cmp::max(edge.tail(), edge.head()).as_usize() + 1;
        if self.edges.len() < needed_len {
            self.edges.resize_with(needed_len, Vec::new);
        }

        let edges = &mut self.edges[edge.tail().as_usize()];
        match edges.binary_search_by_key(&edge.head(), |old_edge| old_edge.head()) {
            Ok(index) => {
                if edge.weight() < edges[index].weight() {
                    edges[index] = *edge;
                }
            }
            Err(index) => edges.insert(index, *edge),
        }
    }

    pub fn remove_edge(&mut self, edge: Edge) -> Option<E> {
        let tail = edge.tail.as_usize();
        let edges = self.edges.get_mut(tail)?;

        edges
            .binary_search_by(|old_edge| old_edge.head().cmp(&edge.head))
            .ok()
            .map(|index| edges.remove(index))
    }
}

impl<E: EdgeLike> GraphLike for AdjacencyListGraph<E> {
    type Edge = E;

    fn outgoing_edges(&self, tail: VertexId) -> &[E] {
        if tail.as_usize() >= self.edges.len() {
            return &[];
        }

        &self.edges[tail.as_usize()]
    }

    fn num_vertices(&self) -> usize {
        self.edges.len()
    }

    fn num_edges(&self) -> usize {
        self.edges.iter().map(|edges| edges.len()).sum()
    }
}
