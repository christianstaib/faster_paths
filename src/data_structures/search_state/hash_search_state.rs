use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    graph::GraphLike,
    types::{Distance, VertexId},
};

use super::search_state_access::SearchStateAccess;

pub struct HashSearchState<D: Distance> {
    distance: FxHashMap<VertexId, D>,
    predecessor: FxHashMap<VertexId, VertexId>,
    expanded: FxHashSet<VertexId>,
}

impl<D: Distance> HashSearchState<D> {
    pub fn new() -> Self {
        Self {
            distance: FxHashMap::default(),
            predecessor: FxHashMap::default(),
            expanded: FxHashSet::default(),
        }
    }
}

impl<D: Distance> SearchStateAccess<D> for HashSearchState<D> {
    fn new<G: GraphLike>(_graph: &G) -> Self {
        Self::new()
    }

    fn get_distance(&self, vertex: VertexId) -> Option<D> {
        self.distance.get(&vertex).copied()
    }

    fn set_distance(&mut self, vertex: VertexId, distance: D) {
        self.distance.insert(vertex, distance);
    }

    fn get_predecessor(&self, vertex: VertexId) -> Option<VertexId> {
        self.predecessor.get(&vertex).copied()
    }

    fn set_predecessor(&mut self, vertex: VertexId, predecessor: VertexId) {
        self.predecessor.insert(vertex, predecessor);
    }

    fn is_expanded(&self, vertex: VertexId) -> bool {
        self.expanded.contains(&vertex)
    }

    fn set_expanded(&mut self, vertex: VertexId) {
        self.expanded.insert(vertex);
    }

    fn clear(&mut self) {
        self.distance = FxHashMap::default();
        self.predecessor = FxHashMap::default();
        self.expanded = FxHashSet::default();
    }
}
