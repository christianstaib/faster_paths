use itertools::Itertools;

use super::{Distance, Edge, Graph, TaillessEdge, Vertex, WeightedEdge};

pub struct VecGraph {
    edges: Vec<TaillessEdge>,
    indices: Vec<(u32, u32)>,
}

impl VecGraph {
    pub fn new(edges: &[WeightedEdge], level_to_vertex: &Vec<Vertex>) -> Self {
        let mut edges_map = edges
            .iter()
            .map(|edge| (edge.tail, edge.remove_tail()))
            .into_group_map();

        let mut edges = Vec::new();
        let mut indices = vec![(0, 0); level_to_vertex.len()];

        for vertex in level_to_vertex {
            let start_index = edges.len() as u32;
            let mut end_index = edges.len() as u32;

            if let Some(mut edges_from_map) = edges_map.remove(vertex) {
                edges_from_map.sort_unstable_by_key(|edge| edge.head);
                end_index += edges_from_map.len() as u32;
                edges.extend(edges_from_map);
            }

            indices[*vertex as usize] = (start_index, end_index);
        }

        Self { edges, indices }
    }
}

impl Graph for VecGraph {
    fn number_of_vertices(&self) -> u32 {
        self.indices.len() as u32
    }

    fn edges(&self, tail: Vertex) -> Box<dyn ExactSizeIterator<Item = WeightedEdge> + Send + '_> {
        let &(start_index, stop_index) = self.indices.get(tail as usize).unwrap_or(&(0, 0));

        struct EdgeIterator<'a> {
            edge_iter: std::slice::Iter<'a, TaillessEdge>,
            tail: Vertex,
        }

        impl<'a> Iterator for EdgeIterator<'a> {
            type Item = WeightedEdge;

            // Returns the next edge in the iterator, setting the tail vertex.
            fn next(&mut self) -> Option<Self::Item> {
                self.edge_iter
                    .next()
                    .map(|tailless_edge| tailless_edge.set_tail(self.tail))
            }
        }

        // Implentig ExactSizeIterator for EdgeIterator
        impl<'a> ExactSizeIterator for EdgeIterator<'a> {
            fn len(&self) -> usize {
                self.edge_iter.len()
            }
        }

        Box::new(EdgeIterator {
            edge_iter: self.edges[start_index as usize..stop_index as usize].iter(),
            tail,
        })
    }

    fn get_weight(&self, edge: &Edge) -> Option<Distance> {
        let &(start_index, stop_index) = self.indices.get(edge.tail as usize).unwrap_or(&(0, 0));
        if let Ok(index) = self.edges[start_index as usize..stop_index as usize]
            .binary_search_by_key(&edge.head, |edge| edge.head)
        {
            return Some(self.edges[index].weight);
        }
        None
    }

    fn set_weight(&mut self, _edge: &Edge, _weight: Option<Distance>) {
        unimplemented!("This is a read only graph");
    }
}
