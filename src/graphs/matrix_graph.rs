use std::usize;

use ahash::HashMapExt;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use serde::{Deserialize, Serialize};

use super::{
    edge::{DirectedEdge, DirectedWeightedEdge},
    Graph, VertexId, Weight,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct MatrixGraph {
    edges: Vec<Vec<Option<Weight>>>, // [tail][head] = Option<Weight>
}

impl Default for MatrixGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph for MatrixGraph {
    fn out_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        struct OutEdgeIterator<'a> {
            source: VertexId,
            current_head: VertexId,
            matrix: &'a Vec<Vec<Option<u32>>>,
        }

        impl<'a> Iterator for OutEdgeIterator<'a> {
            type Item = DirectedWeightedEdge;

            fn next(&mut self) -> Option<Self::Item> {
                while (self.current_head as usize) < self.matrix.len() {
                    let old_current_head = self.current_head;
                    self.current_head += 1;

                    if let Some(weight) =
                        self.matrix[self.source as usize][old_current_head as usize]
                    {
                        return Some(
                            DirectedWeightedEdge::new(self.source, old_current_head, weight)
                                .unwrap(),
                        );
                    }
                }
                None
            }
        }

        impl<'a> ExactSizeIterator for OutEdgeIterator<'a> {
            fn len(&self) -> usize {
                self.matrix
                    .get(self.source as usize)
                    .unwrap()
                    .iter()
                    .flatten()
                    .count()
            }
        }

        let edge_iterator = OutEdgeIterator {
            source,
            current_head: 0,
            matrix: &self.edges,
        };

        Box::new(edge_iterator)
    }

    fn in_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        struct OutEdgeIterator<'a> {
            source: VertexId,
            current_tail: VertexId,
            matrix: &'a Vec<Vec<Option<u32>>>,
        }

        impl<'a> Iterator for OutEdgeIterator<'a> {
            type Item = DirectedWeightedEdge;

            fn next(&mut self) -> Option<Self::Item> {
                while (self.current_tail as usize) < self.matrix.len() {
                    let old_current_tail = self.current_tail;
                    self.current_tail += 1;

                    if let Some(weight) =
                        self.matrix[old_current_tail as usize][self.source as usize]
                    {
                        return Some(
                            DirectedWeightedEdge::new(self.source, old_current_tail, weight)
                                .unwrap(),
                        );
                    }
                }
                None
            }
        }

        impl<'a> ExactSizeIterator for OutEdgeIterator<'a> {
            fn len(&self) -> usize {
                self.matrix
                    .iter()
                    .filter_map(|edges| edges[self.source as usize])
                    .count()
            }
        }

        let edge_iterator = OutEdgeIterator {
            source,
            current_tail: 0,
            matrix: &self.edges,
        };

        Box::new(edge_iterator)
    }

    fn get_edge_weight(&self, edge: &DirectedEdge) -> Option<Weight> {
        if let Some(weight) = self
            .edges
            .get(edge.tail() as usize)?
            .get(edge.head() as usize)?
        {
            return Some(*weight);
        }
        None
    }

    fn number_of_vertices(&self) -> u32 {
        self.edges.len() as u32
    }

    fn number_of_edges(&self) -> u32 {
        self.edges.iter().flatten().flatten().count() as u32
    }

    fn set_edge(&mut self, edge: &DirectedWeightedEdge) {
        let edge_max = std::cmp::max(edge.tail(), edge.head()) + 1;
        if edge_max > self.number_of_vertices() {
            self.edges.resize(edge_max as usize, Vec::new());
            self.edges.shrink_to(self.edges.len() + 1_000);
            for edges in self.edges.iter_mut() {
                edges.resize(edge_max as usize, None);
                edges.shrink_to(edges.len() + 1_000);
            }
        }
        self.add_out_edge(edge);
        self.add_in_edge(edge);
    }

    fn remove_vertex(&mut self, vertex: VertexId) {
        for other_vertex in 0..self.number_of_vertices() {
            self.edges
                .get_mut(vertex as usize)
                .unwrap()
                .get_mut(other_vertex as usize)
                .unwrap()
                .take();
            self.edges
                .get_mut(other_vertex as usize)
                .unwrap()
                .get_mut(vertex as usize)
                .unwrap()
                .take();
        }
    }

    fn set_edges(&mut self, edges: &[DirectedWeightedEdge]) {
        let edges: Vec<_> = edges
            .par_iter()
            // only if new weight is less
            .filter(|edge| {
                edge.weight() < self.get_edge_weight(&edge.unweighted()).unwrap_or(u32::MAX)
            })
            .collect();

        self.edges
            .par_iter_mut()
            .enumerate()
            .for_each(|(tail, out_edges)| {
                edges
                    .iter()
                    .filter(|edge| edge.tail() == tail as u32)
                    .for_each(|edge| {
                        out_edges
                            .get_mut(edge.head() as usize)
                            .unwrap()
                            .insert(edge.weight());
                    })
            });

        self.edges
            .par_iter_mut()
            .enumerate()
            .for_each(|(tail, out_edges)| {
                edges
                    .iter()
                    .filter(|edge| edge.head() == tail as u32)
                    .for_each(|edge| {
                        out_edges
                            .get_mut(edge.tail() as usize)
                            .unwrap()
                            .insert(edge.weight());
                    })
            });
    }
}

impl MatrixGraph {
    pub fn new() -> Self {
        MatrixGraph { edges: Vec::new() }
    }

    pub fn from_edges(edges: &[DirectedWeightedEdge]) -> MatrixGraph {
        let mut graph = MatrixGraph::new();
        edges.iter().for_each(|edge| {
            graph.set_edge(edge);
        });
        graph
    }

    fn add_out_edge(&mut self, edge: &DirectedWeightedEdge) {
        let current_edge = self
            .edges
            .get_mut(edge.tail() as usize)
            .unwrap()
            .get_mut(edge.head() as usize)
            .unwrap();

        if edge.weight() < current_edge.unwrap_or(u32::MAX) {
            *current_edge = Some(edge.weight());
        }
    }

    fn add_in_edge(&mut self, edge: &DirectedWeightedEdge) {
        let current_edge = self
            .edges
            .get_mut(edge.head() as usize)
            .unwrap()
            .get_mut(edge.tail() as usize)
            .unwrap();

        if edge.weight() < current_edge.unwrap_or(u32::MAX) {
            *current_edge = Some(edge.weight());
        }
    }
}
