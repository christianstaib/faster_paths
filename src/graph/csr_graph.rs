use crate::{
    data_structures::FlattenedNested, graph::GraphLike, graph::edge_like::EdgeLike, types::Vertex,
};
use serde::{Deserialize, Serialize};
use std::cmp::max;

/// A graph represented in Compressed Sparse Row (CSR) format.
#[derive(Serialize, Deserialize)]
pub struct CsrGraph<E: EdgeLike> {
    flattened_nested: FlattenedNested<E>,
}

impl<E: EdgeLike> CsrGraph<E> {
    /// Builds a CSR graph from a flat edge list.
    pub fn from_flat(mut flat: Vec<E>) -> Self {
        flat.sort_unstable_by_key(|edge| (edge.tail(), edge.head(), edge.weight()));

        let num_vertices = flat
            .iter()
            .map(|edge| max(edge.tail(), edge.head()))
            .max()
            .map(|vertex| vertex.as_usize() + 1)
            .unwrap_or(0);

        let mut indices = vec![0; num_vertices + 1];

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

    /// Builds a CSR graph from nested outgoing adjacency lists.
    pub fn from_nested(nested: &[Vec<E>]) -> Self
    where
        E: Copy,
    {
        Self {
            flattened_nested: FlattenedNested::from_nested(nested),
        }
    }
}

impl<E: EdgeLike> GraphLike for CsrGraph<E> {
    type Edge = E;

    fn outgoing_edges(&self, tail: Vertex) -> &[E] {
        self.flattened_nested.nested(tail.as_usize())
    }

    fn num_vertices(&self) -> usize {
        self.flattened_nested.num_nested()
    }

    fn num_edges(&self) -> usize {
        self.flattened_nested.num_flat()
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::{CsrGraph, GraphLike, WeightedEdge};
    use crate::types::Vertex;

    #[test]
    fn from_flat_counts_vertices_that_only_appear_as_heads() {
        let graph = CsrGraph::from_flat(vec![WeightedEdge {
            tail: Vertex::from(0),
            head: Vertex::from(10),
            weight: 1_u32,
        }]);

        assert_eq!(graph.num_vertices(), 11);
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn from_flat_empty_graph_has_no_vertices() {
        let graph = CsrGraph::<WeightedEdge<u32>>::from_flat(Vec::new());

        assert_eq!(graph.num_vertices(), 0);
        assert_eq!(graph.num_edges(), 0);
    }
}
