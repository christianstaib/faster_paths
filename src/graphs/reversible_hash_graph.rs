use std::{
    collections::hash_map::{
        Entry::{Occupied, Vacant},
        Iter,
    },
    usize,
};

use ahash::{HashMap, HashMapExt};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use serde::{Deserialize, Serialize};

use super::{
    edge::{DirectedEdge, DirectedWeightedEdge},
    Graph, VertexId, Weight,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ReversibleHashGraph {
    pub out_edges: Vec<HashMap<VertexId, Weight>>,
    pub in_edges: Vec<HashMap<VertexId, Weight>>,
}

impl Default for ReversibleHashGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph for ReversibleHashGraph {
    fn out_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        struct OutEdgeIterator<'a> {
            source: VertexId,
            tailless_edge_iterator: Iter<'a, VertexId, Weight>,
        }

        impl<'a> Iterator for OutEdgeIterator<'a> {
            type Item = DirectedWeightedEdge;

            fn next(&mut self) -> Option<Self::Item> {
                let edge = self.tailless_edge_iterator.next()?;
                Some(DirectedWeightedEdge::new(self.source, *edge.0, *edge.1).unwrap())
            }
        }

        impl<'a> ExactSizeIterator for OutEdgeIterator<'a> {
            fn len(&self) -> usize {
                self.tailless_edge_iterator.len()
            }
        }

        let tailless_edge_iterator = self.out_edges.get(source as usize).unwrap().iter();

        let edge_iterator = OutEdgeIterator {
            source,
            tailless_edge_iterator,
        };

        Box::new(edge_iterator)
    }

    fn in_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        struct InEdgeIterator<'a> {
            source: VertexId,
            tailless_edge_iterator: Iter<'a, VertexId, Weight>,
        }

        impl<'a> Iterator for InEdgeIterator<'a> {
            type Item = DirectedWeightedEdge;

            fn next(&mut self) -> Option<Self::Item> {
                let edge = self.tailless_edge_iterator.next()?;
                Some(DirectedWeightedEdge::new(*edge.0, self.source, *edge.1).unwrap())
            }
        }

        impl<'a> ExactSizeIterator for InEdgeIterator<'a> {
            fn len(&self) -> usize {
                self.tailless_edge_iterator.len()
            }
        }

        let tailless_edge_iterator = self.in_edges.get(source as usize).unwrap().iter();

        let edge_iterator = InEdgeIterator {
            source,
            tailless_edge_iterator,
        };

        Box::new(edge_iterator)
    }

    fn get_edge_weight(&self, edge: &DirectedEdge) -> Option<Weight> {
        self.out_edges
            .get(edge.tail() as usize)?
            .get(&edge.head())
            .cloned()
    }

    fn number_of_vertices(&self) -> u32 {
        self.out_edges.len() as u32
    }

    fn number_of_edges(&self) -> u32 {
        self.out_edges.iter().map(HashMap::len).sum::<usize>() as u32
    }

    fn set_edge(&mut self, edge: &DirectedWeightedEdge) {
        self.add_out_edge(edge);
        self.add_in_edge(edge);
    }

    fn set_edges(&mut self, edges: &[DirectedWeightedEdge]) {
        self.out_edges
            .par_iter_mut()
            .enumerate()
            .for_each(|(tail, out_edges)| {
                edges
                    .iter()
                    .filter(|edge| edge.tail() == tail as u32)
                    .for_each(|edge| {
                        out_edges.insert(edge.head(), edge.weight());
                    })
            });

        self.in_edges
            .par_iter_mut()
            .enumerate()
            .for_each(|(head, in_edges)| {
                edges
                    .iter()
                    .filter(|edge| edge.head() == head as u32)
                    .for_each(|edge| {
                        in_edges.insert(edge.tail(), edge.weight());
                    })
            });
    }

    fn remove_vertex(&mut self, vertex: VertexId) {
        let out_edges = std::mem::take(&mut self.out_edges[vertex as usize]);
        out_edges.iter().for_each(|(&head, &_weight)| {
            if let Some(in_edges) = self.in_edges.get_mut(head as usize) {
                in_edges.remove(&vertex);
            }
        });

        let in_edges = std::mem::take(&mut self.in_edges[vertex as usize]);
        in_edges.iter().for_each(|(&tail, &_weight)| {
            if let Some(out_edges) = self.out_edges.get_mut(tail as usize) {
                out_edges.remove(&vertex);
            }
        });
    }
}

impl ReversibleHashGraph {
    pub fn new() -> Self {
        ReversibleHashGraph {
            out_edges: Vec::new(),
            in_edges: Vec::new(),
        }
    }

    pub fn from_edges(edges: &[DirectedWeightedEdge]) -> ReversibleHashGraph {
        let mut graph = ReversibleHashGraph::new();
        edges.iter().for_each(|edge| {
            graph.set_edge(edge);
        });
        graph
    }

    fn add_out_edge(&mut self, edge: &DirectedWeightedEdge) {
        if (self.out_edges.len() as u32) <= edge.tail() {
            self.out_edges
                .resize((edge.tail() + 1) as usize, HashMap::new());
        }

        match self
            .out_edges
            .get_mut(edge.tail() as usize)
            .unwrap() // after resize it is safe to call unwrap
            .entry(edge.head())
        {
            Occupied(mut o) => {
                let current_weight = o.get_mut();
                if &edge.weight() < current_weight {
                    *current_weight = edge.weight();
                }
            }
            Vacant(v) => {
                v.insert(edge.weight());
            }
        }
    }

    fn add_in_edge(&mut self, edge: &DirectedWeightedEdge) {
        if (self.in_edges.len() as u32) <= edge.head() {
            self.in_edges
                .resize((edge.head() + 1) as usize, HashMap::new());
        }

        match self
            .in_edges
            .get_mut(edge.head() as usize)
            .unwrap() // after resize it is safe to call unwrap
            .entry(edge.tail())
        {
            Occupied(mut o) => {
                let current_weight = o.get_mut();
                if &edge.weight() < current_weight {
                    *current_weight = edge.weight();
                }
            }
            Vacant(v) => {
                v.insert(edge.weight());
            }
        }
    }
}
