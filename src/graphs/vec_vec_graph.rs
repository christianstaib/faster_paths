use super::{Edge, Graph, TaillessEdge, VertexId, Weight, WeightedEdge};

pub struct VecVecGraph {
    pub edges: Vec<Vec<TaillessEdge>>,
}

impl Default for VecVecGraph {
    fn default() -> Self {
        VecVecGraph { edges: Vec::new() }
    }
}

impl Graph for VecVecGraph {
    fn number_of_vertices(&self) -> u32 {
        self.edges.len() as u32
    }

    fn edges(&self, tail: VertexId) -> impl Iterator<Item = WeightedEdge> {
        // Define a struct for iterating over edges with the same tail. Struct is needed
        // as tail would otherwise not live enough.
        struct EdgeIterator<'a> {
            edges: std::slice::Iter<'a, TaillessEdge>,
            tail: VertexId,
        }

        impl<'a> Iterator for EdgeIterator<'a> {
            type Item = WeightedEdge;

            // Returns the next edge in the iterator, setting the tail vertex.
            fn next(&mut self) -> Option<Self::Item> {
                self.edges
                    .next()
                    .map(|tailless_edge| tailless_edge.set_tail(self.tail))
            }
        }

        EdgeIterator {
            edges: self.edges[tail as usize].iter(),
            tail,
        }
    }

    fn get_weight(&self, edge: &Edge) -> Option<Weight> {
        // Retrieve the vector of edges sharing the same tail, if it exists.
        let edges_sharing_tail = self.edges.get(edge.tail as usize)?;

        // Perform a binary search to find the index of the edge with the same head.
        let edge_index = edges_sharing_tail
            .binary_search_by_key(&edge.head, |tailless_edge| tailless_edge.head)
            .ok()?;

        // Return the weight of the found edge.
        Some(edges_sharing_tail[edge_index].weight)
    }

    fn set_weight(&mut self, edge: &Edge, weight: Option<Weight>) {
        // Ensure the tail index is within the bounds of self.edges.
        if edge.tail as usize >= self.edges.len() {
            self.edges.resize(edge.tail as usize + 1, Vec::new());
        }

        // Get a mutable reference to the vector of edges sharing the same tail.
        let edges_sharing_tail = &mut self.edges[edge.tail as usize];

        // Find the index of the edge in edges_sharing_tail with the same head.
        let edge_index = edges_sharing_tail.binary_search_by_key(&edge.head, |other| other.head);

        if let Some(weight) = weight {
            // If a weight is provided, connect or update the edge.
            match edge_index {
                Ok(index) => {
                    // Edge exists, update its weight if the new weight is smaller.
                    if weight < edges_sharing_tail[index].weight {
                        edges_sharing_tail[index].weight = weight;
                    }
                }
                Err(index) => {
                    // Edge doesn't exist, insert the new edge.
                    let new_edge = TaillessEdge {
                        head: edge.head,
                        weight,
                    };
                    edges_sharing_tail.insert(index, new_edge);
                }
            }
        } else {
            // If no weight is provided, disconnect the edge.
            if let Ok(index) = edge_index {
                edges_sharing_tail.remove(index);
            }
        }
    }
}
