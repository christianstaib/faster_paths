use crate::{
    graph::GraphLike,
    types::{Distance, VertexId},
};

use super::search_state_access::SearchStateAccess;

pub struct VecSearchState<D: Distance> {
    distance: Vec<D>,
    predecessor: Vec<VertexId>,
    expanded: Vec<bool>,
}

impl<D: Distance> VecSearchState<D> {
    pub fn new(num_vertices: usize) -> Self {
        Self {
            distance: vec![D::max_value(); num_vertices],
            predecessor: vec![VertexId::new(u32::MAX); num_vertices],
            expanded: vec![false; num_vertices],
        }
    }
}

impl<D: Distance> SearchStateAccess<D> for VecSearchState<D> {
    fn new<G: GraphLike>(graph: &G) -> Self {
        Self::new(graph.num_vertices())
    }

    fn get_distance(&self, vertex: VertexId) -> Option<D> {
        let distance = self.distance.get(vertex.as_usize()).copied()?;
        (distance != D::max_value()).then_some(distance)
    }

    fn set_distance(&mut self, vertex: VertexId, distance: D) {
        self.distance[vertex.as_usize()] = distance;
    }

    fn get_predecessor(&self, vertex: VertexId) -> Option<VertexId> {
        let predecessor = self.predecessor.get(vertex.as_usize()).copied()?;
        (predecessor != VertexId::new(u32::MAX)).then_some(predecessor)
    }

    fn set_predecessor(&mut self, vertex: VertexId, predecessor: VertexId) {
        self.predecessor[vertex.as_usize()] = predecessor;
    }

    fn is_expanded(&self, vertex: VertexId) -> bool {
        self.expanded
            .get(vertex.as_usize())
            .copied()
            .unwrap_or(false)
    }

    fn set_expanded(&mut self, vertex: VertexId) {
        self.expanded[vertex.as_usize()] = true;
    }

    fn clear(&mut self) {
        self.distance.fill(D::max_value());
        self.predecessor.fill(VertexId::new(u32::MAX));
        self.expanded.fill(false);
    }
}
