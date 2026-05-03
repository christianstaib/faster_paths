use std::collections::{HashMap, HashSet};

use crate::{
    search_state::search_state_access::SearchStateAccess,
    types::{Distance, VertexId},
};

pub struct HashSearchState {
    distance: HashMap<VertexId, Distance>,
    predecessor: HashMap<VertexId, VertexId>,
    expanded: HashSet<VertexId>,
}

impl HashSearchState {
    pub fn new() -> Self {
        Self {
            distance: HashMap::new(),
            predecessor: HashMap::new(),
            expanded: HashSet::new(),
        }
    }
}

impl SearchStateAccess for HashSearchState {
    fn get_distance(&self, vertex: VertexId) -> Option<Distance> {
        self.distance.get(&vertex).copied()
    }

    fn set_distance(&mut self, vertex: VertexId, distance: Distance) {
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
        self.distance = HashMap::new();
        self.predecessor = HashMap::new();
        self.expanded = HashSet::new();
    }
}
