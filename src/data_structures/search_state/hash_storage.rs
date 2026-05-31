use rustc_hash::{FxHashMap, FxHashSet};

use crate::types::Vertex;

use super::storage::{VertexMap, VertexSet};

pub struct HashVertexMap<T> {
    values: FxHashMap<Vertex, T>,
}

impl<T> Default for HashVertexMap<T> {
    fn default() -> Self {
        Self {
            values: FxHashMap::default(),
        }
    }
}

impl<T: Copy> VertexMap<T> for HashVertexMap<T> {
    fn new(_len: usize, _default: T) -> Self {
        Self::default()
    }

    fn get(&self, vertex: Vertex) -> Option<T> {
        self.values.get(&vertex).copied()
    }

    fn set(&mut self, vertex: Vertex, value: T) {
        self.values.insert(vertex, value);
    }

    fn clear(&mut self) {
        self.values.clear();
    }
}

pub struct HashVertexSet {
    values: FxHashSet<Vertex>,
}

impl Default for HashVertexSet {
    fn default() -> Self {
        Self {
            values: FxHashSet::default(),
        }
    }
}

impl VertexSet for HashVertexSet {
    fn new(_len: usize) -> Self {
        Self::default()
    }

    fn contains(&self, vertex: Vertex) -> bool {
        self.values.contains(&vertex)
    }

    fn insert(&mut self, vertex: Vertex) {
        self.values.insert(vertex);
    }

    fn clear(&mut self) {
        self.values.clear();
    }
}
