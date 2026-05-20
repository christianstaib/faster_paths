use std::{
    fmt::Debug,
    ops::{Add, AddAssign},
};

use num_traits::{Bounded, Zero};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct VertexId(u32);

impl VertexId {
    pub fn new(vertex_id: u32) -> Self {
        Self(vertex_id)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

pub trait Distance:
    Copy + Ord + Zero + Bounded + Add<Output = Self> + AddAssign + Debug + Send + Sync
{
}

impl<D> Distance for D where
    D: Copy + Ord + Zero + Bounded + Add<Output = Self> + AddAssign + Debug + Send + Sync
{
}
