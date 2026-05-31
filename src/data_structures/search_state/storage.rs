use crate::types::Vertex;

pub trait VertexMap<T: Copy> {
    fn new(len: usize, default: T) -> Self;
    fn get(&self, vertex: Vertex) -> Option<T>;
    fn set(&mut self, vertex: Vertex, value: T);
    fn clear(&mut self);
}

pub trait VertexSet {
    fn new(len: usize) -> Self;
    fn contains(&self, vertex: Vertex) -> bool;
    fn insert(&mut self, vertex: Vertex);
    fn clear(&mut self);

    fn contains_and_insert(&mut self, vertex: Vertex) -> bool {
        let contains = self.contains(vertex);
        self.insert(vertex);
        contains
    }
}

pub fn reversed_path(predecessor: &impl VertexMap<Vertex>, target: Vertex) -> Vec<Vertex> {
    let mut path = vec![target];

    let mut current = target;
    while let Some(previous) = predecessor.get(current) {
        current = previous;
        path.push(current);
    }

    path
}
