use crate::types::Vertex;

use super::storage::{VertexMap, VertexSet};

pub struct VecVertexMap<T> {
    values: Vec<T>,
    default: T,
}

impl<T> VertexMap<T> for VecVertexMap<T>
where
    T: Copy + PartialEq,
{
    fn new(len: usize, default: T) -> Self {
        Self {
            values: vec![default; len],
            default,
        }
    }

    fn get(&self, vertex: Vertex) -> Option<T> {
        let value = self.values.get(vertex as usize).copied()?;
        (value != self.default).then_some(value)
    }

    fn set(&mut self, vertex: Vertex, value: T) {
        self.values[vertex as usize] = value;
    }

    fn clear(&mut self) {
        self.values.fill(self.default);
    }
}

pub struct VecVertexSet {
    values: Vec<bool>,
}

impl VertexSet for VecVertexSet {
    fn new(len: usize) -> Self {
        Self {
            values: vec![false; len],
        }
    }

    fn contains(&self, vertex: Vertex) -> bool {
        self.values.get(vertex as usize).copied().unwrap_or(false)
    }

    fn insert(&mut self, vertex: Vertex) {
        self.values[vertex as usize] = true;
    }

    fn clear(&mut self) {
        self.values.fill(false);
    }
}
