use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlattenedNested<T> {
    flat: Vec<T>,
    offsets: Vec<usize>,
}

impl<T> FlattenedNested<T> {
    pub fn from_flat(flat: Vec<T>, offsets: Vec<usize>) -> Self {
        Self { flat, offsets }
    }

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
