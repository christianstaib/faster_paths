use crate::graph::EdgeLike;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlattenedNested<T> {
    flat: Vec<T>,
    offsets: Vec<usize>,
}

impl<T> FlattenedNested<T> {
    pub fn new(nested: &Vec<Vec<T>>) -> Self
    where
        T: Copy,
    {
        let total = nested.iter().map(Vec::len).sum();

        let mut flat = Vec::with_capacity(total);
        let mut offsets = Vec::with_capacity(nested.len() + 1);

        offsets.push(0);

        for inner in nested {
            flat.extend(inner);
            offsets.push(flat.len());
        }

        Self { flat, offsets }
    }

    pub fn nested(&self, index: usize) -> &[T] {
        if index >= self.num_nested() {
            return &self.flat[0..0];
        }

        let begin = self.offsets[index];
        let end = self.offsets[index + 1];

        &self.flat[begin..end]
    }

    pub fn num_nested(&self) -> usize {
        self.offsets.len().saturating_sub(1)
    }

    pub fn num_flat(&self) -> usize {
        self.flat.len()
    }
}

impl<T: EdgeLike> FlattenedNested<T> {
    pub fn from_flat(mut flat: Vec<T>) -> Self {
        let num_vertices = flat
            .iter()
            .map(|edge| edge.tail().max(edge.head()).as_usize())
            .max()
            .map_or(0, |vertex| vertex + 1);

        flat.sort_unstable_by_key(|edge| (edge.tail(), edge.head()));

        let mut offsets = Vec::with_capacity(num_vertices + 1);
        let mut edge_index = 0;

        for vertex in 0..num_vertices {
            offsets.push(edge_index);

            while edge_index < flat.len() && flat[edge_index].tail().as_usize() == vertex {
                edge_index += 1;
            }
        }

        offsets.push(flat.len());

        Self { flat, offsets }
    }
}
