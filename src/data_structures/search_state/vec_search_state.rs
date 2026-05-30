use crate::{
    graph::GraphLike,
    types::{Distance, Vertex},
};

use super::search_state_access::SearchStateAccess;

pub struct VecSearchState<D: Distance> {
    distance: Vec<D>,
    predecessor: Vec<Vertex>,
    expanded: Vec<bool>,
}

impl<D: Distance> VecSearchState<D> {
    pub fn new(num_vertices: usize) -> Self {
        Self {
            distance: vec![D::max_value(); num_vertices],
            predecessor: vec![u32::MAX; num_vertices],
            expanded: vec![false; num_vertices],
        }
    }
}

impl<D: Distance> SearchStateAccess<D> for VecSearchState<D> {
    fn new<G: GraphLike>(graph: &G) -> Self {
        Self::new(graph.num_vertices())
    }

    fn get_distance(&self, vertex: Vertex) -> Option<D> {
        let distance = self.distance.get(vertex as usize).copied()?;
        (distance != D::max_value()).then_some(distance)
    }

    fn set_distance(&mut self, vertex: Vertex, distance: D) {
        self.distance[vertex as usize] = distance;
    }

    fn get_predecessor(&self, vertex: Vertex) -> Option<Vertex> {
        let predecessor = self.predecessor.get(vertex as usize).copied()?;
        (predecessor != u32::MAX).then_some(predecessor)
    }

    fn set_predecessor(&mut self, vertex: Vertex, predecessor: Vertex) {
        self.predecessor[vertex as usize] = predecessor;
    }

    fn is_expanded(&self, vertex: Vertex) -> bool {
        self.expanded
            .get(vertex as usize)
            .copied()
            .unwrap_or(false)
    }

    fn set_expanded(&mut self, vertex: Vertex) {
        self.expanded[vertex as usize] = true;
    }

    fn clear(&mut self) {
        self.distance.fill(D::max_value());
        self.predecessor.fill(u32::MAX);
        self.expanded.fill(false);
    }
}
