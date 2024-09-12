use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{Distance, Edge, Graph, TaillessEdge, Vertex, WeightedEdge};

#[derive(Clone, Serialize, Deserialize)]
pub struct VecHashGraph {
    edges: Vec<HashMap<Vertex, Distance>>,
}

impl Default for VecHashGraph {
    fn default() -> Self {
        VecHashGraph { edges: Vec::new() }
    }
}

impl VecHashGraph {
    pub fn from_edges(edges: &Vec<WeightedEdge>) -> VecHashGraph {
        let mut graph = VecHashGraph::default();

        edges.iter().for_each(|edge| {
            if edge.weight
                < graph
                    .get_weight(&edge.remove_weight())
                    .unwrap_or(Distance::MAX)
            {
                graph.set_weight(&edge.remove_weight(), Some(edge.weight));
            }
        });

        graph
    }
}

impl Graph for VecHashGraph {
    fn number_of_vertices(&self) -> u32 {
        self.edges.len() as u32
    }

    fn edges(&self, tail: Vertex) -> Box<dyn ExactSizeIterator<Item = WeightedEdge> + Send + '_> {
        // Define a struct for iterating over edges with the same tail. Struct is needed
        // as tail would otherwise not live enough.
        struct EdgeIterator<'a> {
            edge_iter: std::collections::hash_map::Iter<'a, u32, u32>,
            tail: Vertex,
        }

        impl<'a> Iterator for EdgeIterator<'a> {
            type Item = WeightedEdge;

            // Returns the next edge in the iterator, setting the tail vertex.
            fn next(&mut self) -> Option<Self::Item> {
                self.edge_iter
                    .next()
                    .map(|(&head, &weight)| WeightedEdge::new(self.tail, head, weight))
            }
        }

        // Implentig ExactSizeIterator for EdgeIterator
        impl<'a> ExactSizeIterator for EdgeIterator<'a> {
            fn len(&self) -> usize {
                self.edge_iter.len()
            }
        }

        Box::new(EdgeIterator {
            edge_iter: self.edges[tail as usize].iter(),
            tail,
        })
    }

    fn get_weight(&self, edge: &Edge) -> Option<Distance> {
        // Retrieve the vector of edges sharing the same tail, if it exists.
        let edges_sharing_tail = self.edges.get(edge.tail as usize)?;

        edges_sharing_tail.get(&edge.head).cloned()
    }

    fn set_weight(&mut self, edge: &Edge, weight: Option<Distance>) {
        // Ensure the edge endpoints is within the bounds of self.edges.
        let max_edge_endpoints = std::cmp::max(edge.tail, edge.head) as usize;
        if max_edge_endpoints >= self.edges.len() {
            self.edges.resize(max_edge_endpoints + 1, HashMap::new());
        }

        // Get a mutable reference to the vector of edges sharing the same tail.
        let edges_sharing_tail = &mut self.edges[edge.tail as usize];

        if weight.is_none() {
            edges_sharing_tail.remove(&edge.head);
        } else {
            edges_sharing_tail.insert(edge.head, weight.unwrap());
        }
    }
}
