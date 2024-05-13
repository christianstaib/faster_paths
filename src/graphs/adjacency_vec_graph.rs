use std::slice::Iter;

use itertools::Itertools;

use super::{
    edge::{DirectedTaillessWeightedEdge, DirectedWeightedEdge},
    Graph, VertexId,
};

/// Graph that is optimized for cache efficency
pub struct AdjacencyVecGraph {
    edges: Vec<DirectedTaillessWeightedEdge>,
    indices: Vec<(u32, u32)>, // (start, end)
}

impl AdjacencyVecGraph {
    pub fn new(edges: &[DirectedWeightedEdge], order: &[VertexId]) -> Self {
        let mut edges_map = edges
            .iter()
            .map(|edge| (edge.tail(), edge.tailless()))
            .into_group_map();
        let mut edges = Vec::new();
        let mut indices = Vec::new();
        for vertex in order {
            if let Some(edges_from_map) = edges_map.remove(vertex) {
                indices.push((
                    indices.len() as u32,
                    (indices.len() + edges_from_map.len()) as u32,
                ));
                edges.extend(edges_from_map);
            } else {
                indices.push((indices.len() as u32, indices.len() as u32));
            }
        }
        Self { edges, indices }
    }
}

impl Graph for AdjacencyVecGraph {
    fn out_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        struct OutEdgeIterator<'a> {
            source: VertexId,
            tailless_edge_iterator: Iter<'a, DirectedTaillessWeightedEdge>,
        }

        impl<'a> Iterator for OutEdgeIterator<'a> {
            type Item = DirectedWeightedEdge;

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
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        unimplemented!("cannot visits incoming edges");
    }

    fn number_of_vertices(&self) -> u32 {
        self.indices.len() as u32
    }

    fn number_of_edges(&self) -> u32 {
        self.edges.len() as u32
    }

    fn set_edge(&mut self, _edge: &DirectedWeightedEdge) {
        unimplemented!("this graph cannot be mutated");
    }

    fn remove_vertex(&mut self, _vertex: VertexId) {
        unimplemented!("this graph cannot be mutated");
    }
}
