use std::slice::Iter;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    edge::{TaillessWeightedEdge, WeightedEdge},
    Graph, VertexId,
};

/// Graph that is optimized for cache efficency
#[derive(Serialize, Deserialize, Clone)]
pub struct AdjacencyVecGraph {
    edges: Vec<TaillessWeightedEdge>,
    indices: Vec<(u32, u32)>, // (start, end)
}

impl AdjacencyVecGraph {
    pub fn new(edges: &[WeightedEdge], order: &[VertexId]) -> Self {
        let mut edges_map = edges
            .iter()
            .map(|edge| (edge.tail(), edge.tailless()))
            .into_group_map();

        let mut edges = Vec::new();
        let mut indices = vec![(0, 0); order.len()];

        for vertex in order {
            let start_index = edges.len() as u32;
            let mut end_index = edges.len() as u32;

            if let Some(edges_from_map) = edges_map.remove(vertex) {
                end_index += edges_from_map.len() as u32;
                edges.extend(edges_from_map);
            }

            indices[*vertex as usize] = (start_index, end_index);
        }

        Self { edges, indices }
    }
}

impl Graph for AdjacencyVecGraph {
    fn out_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = WeightedEdge> + Send + '_> {
        struct OutEdgeIterator<'a> {
            source: VertexId,
            tailless_edge_iterator: Iter<'a, TaillessWeightedEdge>,
        }

        impl<'a> Iterator for OutEdgeIterator<'a> {
            type Item = WeightedEdge;

            fn next(&mut self) -> Option<Self::Item> {
                let edge = self.tailless_edge_iterator.next()?;
                edge.set_tail(self.source)
            }
        }

        impl<'a> ExactSizeIterator for OutEdgeIterator<'a> {
            fn len(&self) -> usize {
                self.tailless_edge_iterator.len()
            }
        }

        let tailless_edge_iterator = self.edges[(self.indices[source as usize].0 as usize)
            ..(self.indices[source as usize].1 as usize)]
            .iter();

        let edge_iterator = OutEdgeIterator {
            source,
            tailless_edge_iterator,
        };

        Box::new(edge_iterator)
    }

    fn in_edges(
        &self,
        _source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = WeightedEdge> + Send + '_> {
        unimplemented!("cannot visits incoming edges");
    }

    fn number_of_vertices(&self) -> u32 {
        self.indices.len() as u32
    }

    fn number_of_edges(&self) -> u32 {
        self.edges.len() as u32
    }

    fn set_edge(&mut self, _edge: &WeightedEdge) {
        unimplemented!("this graph cannot be mutated");
    }

    fn remove_vertex(&mut self, _vertex: VertexId) {
        unimplemented!("this graph cannot be mutated");
    }
}
