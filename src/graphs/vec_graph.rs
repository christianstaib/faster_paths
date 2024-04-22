use std::{slice::Iter, usize};

use serde::{Deserialize, Serialize};

use super::{
    edge::{DirectedTaillessWeightedEdge, DirectedWeightedEdge},
    Graph, VertexId,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct VecGraph {
    pub edges: Vec<Vec<DirectedTaillessWeightedEdge>>,
}

impl Default for VecGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph for VecGraph {
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

        let tailless_edge_iterator = if let Some(edges) = self.edges.get(source as usize) {
            edges.iter()
        } else {
            [].iter()
        };

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
        unimplemented!("this graph is not reversable");
    }

    fn number_of_vertices(&self) -> u32 {
        self.edges.len() as u32
    }

    fn number_of_edges(&self) -> u32 {
        self.edges.iter().map(Vec::len).sum::<usize>() as u32
    }

    fn set_edge(&mut self, edge: &DirectedWeightedEdge) {
        if (self.edges.len() as u32) <= edge.tail() {
            self.edges.resize((edge.tail() + 1) as usize, Vec::new());
        }

        match self.edges[edge.tail() as usize]
            .binary_search_by_key(&edge.head(), |out_edge| out_edge.head())
        {
            Ok(idx) => {
                if edge.weight() < self.edges[edge.tail() as usize][idx].weight() {
                    self.edges[edge.tail() as usize][idx].set_weight(edge.weight());
                }
            }
            Err(idx) => self.edges[edge.tail() as usize].insert(idx, edge.tailless()),
        }
    }

    fn remove_vertex(&mut self, vertex: VertexId) {
        if let Some(edges) = self.edges.get_mut(vertex as usize) {
            edges.clear();
        }
    }
}

impl VecGraph {
    pub fn new() -> Self {
        VecGraph { edges: Vec::new() }
    }

    pub fn from_edges(edges: &[DirectedWeightedEdge]) -> VecGraph {
        let mut graph = VecGraph::new();
        edges.iter().for_each(|edge| {
            graph.set_edge(edge);
        });
        graph
    }
}
