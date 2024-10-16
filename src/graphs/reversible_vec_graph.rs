use std::{slice::Iter, usize};

use indicatif::ProgressIterator;
use serde::{Deserialize, Serialize};

use super::{
    edge::{HeadlessWeightedEdge, TaillessWeightedEdge, WeightedEdge},
    Graph, VertexId,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ReversibleVecGraph {
    pub out_edges: Vec<Vec<TaillessWeightedEdge>>,
    pub in_edges: Vec<Vec<HeadlessWeightedEdge>>,
    pub number_of_vertices: u32,
}

impl Default for ReversibleVecGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph for ReversibleVecGraph {
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
                Some(edge.set_tail(self.source).unwrap())
            }
        }

        impl<'a> ExactSizeIterator for OutEdgeIterator<'a> {
            fn len(&self) -> usize {
                self.tailless_edge_iterator.len()
            }
        }

        let tailless_edge_iterator = if let Some(edges) = self.out_edges.get(source as usize) {
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

    fn number_of_vertices(&self) -> u32 {
        self.number_of_vertices
    }

    fn number_of_edges(&self) -> u32 {
        self.out_edges.iter().map(Vec::len).sum::<usize>() as u32
    }

    fn in_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = WeightedEdge> + Send + '_> {
        struct InEdgeIterator<'a> {
            source: VertexId,
            tailless_edge_iterator: Iter<'a, HeadlessWeightedEdge>,
        }

        impl<'a> Iterator for InEdgeIterator<'a> {
            type Item = WeightedEdge;

            fn next(&mut self) -> Option<Self::Item> {
                let edge = self.tailless_edge_iterator.next()?;
                Some(edge.set_head(self.source).unwrap())
            }
        }

        impl<'a> ExactSizeIterator for InEdgeIterator<'a> {
            fn len(&self) -> usize {
                self.tailless_edge_iterator.len()
            }
        }

        let tailless_edge_iterator = if let Some(edges) = self.in_edges.get(source as usize) {
            edges.iter()
        } else {
            [].iter()
        };

        let edge_iterator = InEdgeIterator {
            source,
            tailless_edge_iterator,
        };

        Box::new(edge_iterator)
    }

    fn set_edge(&mut self, edge: &WeightedEdge) {
        self.number_of_vertices = *[edge.tail() + 1, edge.head() + 1, self.number_of_vertices]
            .iter()
            .max()
            .unwrap();
        self.add_out_edge(edge);
        self.add_in_edge(edge);
    }

    fn remove_vertex(&mut self, vertex: VertexId) {
        let out_edges = std::mem::take(&mut self.out_edges[vertex as usize]);
        out_edges.iter().for_each(|out_edge| {
            if let Ok(idx) = self.in_edges[out_edge.head() as usize]
                .binary_search_by_key(&vertex, |in_edge| in_edge.tail())
            {
                self.in_edges[out_edge.head() as usize].remove(idx);
            }
        });

        let in_edges = std::mem::take(&mut self.in_edges[vertex as usize]);
        in_edges.iter().for_each(|in_edge| {
            if let Ok(idx) = self.out_edges[in_edge.tail() as usize]
                .binary_search_by_key(&vertex, |in_edge| in_edge.head())
            {
                self.out_edges[in_edge.tail() as usize].remove(idx);
            }
        });
    }
}

impl ReversibleVecGraph {
    pub fn new() -> Self {
        ReversibleVecGraph {
            out_edges: Vec::new(),
            in_edges: Vec::new(),
            number_of_vertices: 0,
        }
    }

    pub fn from_edges(edges: &[WeightedEdge]) -> ReversibleVecGraph {
        let mut graph = ReversibleVecGraph::new();
        edges.iter().progress().for_each(|edge| {
            graph.set_edge(edge);
        });
        graph
    }

    fn add_out_edge(&mut self, edge: &WeightedEdge) {
        if (self.out_edges.len() as u32) <= edge.tail() {
            self.out_edges
                .resize((edge.tail() + 1) as usize, Vec::new());
        }

        match self.out_edges[edge.tail() as usize]
            .binary_search_by_key(&edge.head(), |out_edge| out_edge.head())
        {
            Ok(idx) => {
                if edge.weight() < self.out_edges[edge.tail() as usize][idx].weight() {
                    self.out_edges[edge.tail() as usize][idx].set_weight(edge.weight());
                }
            }
            Err(idx) => self.out_edges[edge.tail() as usize].insert(idx, edge.tailless()),
        }
    }

    fn add_in_edge(&mut self, edge: &WeightedEdge) {
        if (self.in_edges.len() as u32) <= edge.head() {
            self.in_edges.resize((edge.head() + 1) as usize, Vec::new());
        }

        match self.in_edges[edge.head() as usize]
            .binary_search_by_key(&edge.tail(), |out_edge| out_edge.tail())
        {
            Ok(idx) => {
                if edge.weight() < self.in_edges[edge.head() as usize][idx].weight() {
                    self.in_edges[edge.head() as usize][idx].set_weight(edge.weight());
                }
            }
            Err(idx) => self.in_edges[edge.head() as usize].insert(idx, edge.headless()),
        }
    }
}
