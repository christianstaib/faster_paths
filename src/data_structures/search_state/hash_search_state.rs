use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    graph::GraphLike,
    types::{Distance, Vertex},
};

use super::search_state_access::SearchStateAccess;

pub struct HashSearchState<D: Distance> {
    distance: FxHashMap<Vertex, D>,
    predecessor: FxHashMap<Vertex, Vertex>,
    expanded: FxHashSet<Vertex>,
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

impl<D: Distance> Default for HashSearchState<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D: Distance> SearchStateAccess<D> for HashSearchState<D> {
    fn new<G: GraphLike>(_graph: &G) -> Self {
        Self::new()
    }

    fn get_distance(&self, vertex: Vertex) -> Option<D> {
        self.distance.get(&vertex).copied()
    }

    fn set_distance(&mut self, vertex: Vertex, distance: D) {
        self.distance.insert(vertex, distance);
    }

    fn get_predecessor(&self, vertex: Vertex) -> Option<Vertex> {
        self.predecessor.get(&vertex).copied()
    }

    fn set_predecessor(&mut self, vertex: Vertex, predecessor: Vertex) {
        self.predecessor.insert(vertex, predecessor);
    }

    fn is_expanded(&self, vertex: Vertex) -> bool {
        self.expanded.contains(&vertex)
    }

    fn set_expanded(&mut self, vertex: Vertex) {
        self.expanded.insert(vertex);
    }

    fn clear(&mut self) {
        self.distance = FxHashMap::default();
        self.predecessor = FxHashMap::default();
        self.expanded = FxHashSet::default();
    }
}
