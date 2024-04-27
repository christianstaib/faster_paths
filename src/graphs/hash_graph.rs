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

use super::{edge::DirectedWeightedEdge, graph_functions::all_edges, Graph, VertexId, Weight};

#[derive(Clone, Serialize, Deserialize)]
pub struct HashGraph {
    pub out_edges: Vec<HashMap<VertexId, Weight>>,
}

impl Default for HashGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph for HashGraph {
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
        _source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_> {
        unimplemented!("this graph is not reversable");
    }

    fn get_edge_weight(&self, edge: &super::edge::DirectedEdge) -> Option<Weight> {
        Some(
            *self
                .out_edges
                .get(edge.tail() as usize)?
                .get(&edge.head())?,
        )
    }

    fn number_of_vertices(&self) -> u32 {
        self.out_edges.len() as u32
    }

    fn number_of_edges(&self) -> u32 {
        self.out_edges.iter().map(HashMap::len).sum::<usize>() as u32
    }

    fn set_edge(&mut self, edge: &DirectedWeightedEdge) {
        self.add_out_edge(edge);
    }

    fn set_edges(&mut self, edges: &[DirectedWeightedEdge]) {
        let edges: Vec<_> = edges
            .par_iter()
            // only if new weight is less
            .filter(|edge| {
                edge.weight() < self.get_edge_weight(&edge.unweighted()).unwrap_or(u32::MAX)
            })
            .collect();

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
    }

    fn remove_vertex(&mut self, vertex: VertexId) {
        std::mem::take(&mut self.out_edges[vertex as usize]);
    }
}

impl HashGraph {
    pub fn new() -> Self {
        HashGraph {
            out_edges: Vec::new(),
        }
    }

    pub fn from_graph(graph: &dyn Graph) -> Self {
        let mut hash_graph = HashGraph::new();
        all_edges(graph).iter().for_each(|edge| {
            hash_graph.set_edge(edge);
        });
        hash_graph
    }

    pub fn from_edges(edges: &[DirectedWeightedEdge]) -> HashGraph {
        let mut graph = HashGraph::new();
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
}
