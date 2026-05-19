use crate::{
    graph::{Edge, GraphLike, edge_like::EdgeLike},
    types::VertexId,
};

/// A graph represented by adjacency lists.
pub struct Graph<E: EdgeLike> {
    edges: Vec<Vec<E>>,
}

impl<E: EdgeLike> Graph<E> {
    pub fn from_nested(edges: Vec<Vec<E>>) -> Self {
        Self { edges }
    }

    pub fn out_edges(&self, tail: VertexId) -> &[E] {
        if tail.as_usize() >= self.edges.len() {
            return &[];
        }

        &self.edges[tail.as_usize()]
    }

    pub fn add_edge(&mut self, edge: E) {
        let tail = edge.tail();
        let head = edge.head();
        self.ensure_vertices(std::cmp::max(tail, head).as_usize() + 1);

        let edges = &mut self.edges[tail.as_usize()];
        match edges.binary_search_by(|old_edge| old_edge.head().cmp(&head)) {
            Ok(index) => {
                edges[index] = edge;
            }
            Err(index) => edges.insert(index, edge),
        }
    }

    pub fn remove_edge(&mut self, edge: Edge) -> Option<E> {
        let tail = edge.tail.as_usize();
        let edges = self.edges.get_mut(tail)?;

        edges
            .binary_search_by(|old_edge| old_edge.head().cmp(&edge.head))
            .ok()
            .map(|index| edges.remove(index))
    }

    fn ensure_vertices(&mut self, num_vertices: usize) {
        if self.edges.len() < num_vertices {
            self.edges.resize_with(num_vertices, Vec::new);
        }
    }
}

impl<E: EdgeLike> GraphLike for Graph<E> {
    type Edge = E;

    fn out_edges(&self, tail: VertexId) -> &[E] {
        self.out_edges(tail)
    }

    fn num_vertices(&self) -> usize {
        self.edges.len()
    }

    fn num_edges(&self) -> usize {
        self.edges.iter().map(|edges| edges.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        graph::{Edge, Graph, GraphLike, WeightedEdge},
        types::VertexId,
    };

    fn vertex(id: u32) -> VertexId {
        VertexId::new(id)
    }

    fn edge(tail: u32, head: u32, weight: u32) -> WeightedEdge<u32> {
        WeightedEdge {
            tail: vertex(tail),
            head: vertex(head),
            weight,
        }
    }

    #[test]
    fn add_edge_keeps_out_edges_sorted_and_keeps_shortest_duplicate() {
        let mut graph = Graph::from_nested(Vec::new());

        graph.add_edge(edge(0, 2, 20));
        graph.add_edge(edge(0, 1, 10));
        graph.add_edge(edge(0, 2, 15));
        graph.add_edge(edge(0, 2, 30));

        let edges = graph.out_edges(vertex(0));
        assert_eq!(edges.len(), 2);
        assert_eq!(edges[0].head, vertex(1));
        assert_eq!(edges[0].weight, 10);
        assert_eq!(edges[1].head, vertex(2));
        assert_eq!(edges[1].weight, 15);
    }

    #[test]
    fn remove_edge_removes_the_matching_tail_and_head() {
        let mut graph = Graph::from_nested(Vec::new());

        graph.add_edge(edge(0, 1, 10));
        graph.add_edge(edge(1, 2, 20));

        let removed = graph.remove_edge(Edge {
            tail: vertex(0),
            head: vertex(1),
        });

        assert!(removed.is_some());
        assert!(graph.out_edges(vertex(0)).is_empty());
        assert_eq!(graph.out_edges(vertex(1)).len(), 1);
    }

    #[test]
    fn add_edge_does_not_shrink_existing_vertices() {
        let mut graph = Graph::from_nested(vec![Vec::new(), Vec::new(), Vec::new()]);

        graph.add_edge(edge(0, 1, 10));

        assert_eq!(graph.num_vertices(), 3);
    }
}
