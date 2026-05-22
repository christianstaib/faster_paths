use std::{
    fmt::Debug,
    num::ParseIntError,
    ops::{Add, AddAssign, Sub},
    str::FromStr,
};

use num_traits::{Bounded, Zero};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Vertex(u32);

impl From<u32> for Vertex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Vertex {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl FromStr for Vertex {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u32::from_str(s).map(Vertex)
    }
}

pub trait Distance:
    Copy
    + Ord
    + Zero
    + Bounded
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + Debug
    + Send
    + Sync
{
}

impl<D> Distance for D where
    D: Copy
        + Ord
        + Zero
        + Bounded
        + Add<Output = Self>
        + AddAssign
        + Sub<Output = Self>
        + Debug
        + Send
        + Sync
{
}

/// Checks whether the absolute difference between `left` and `right` is at most `epsilon`.
pub fn distance_abs_diff_eq<D>(left: D, right: D, epsilon: D) -> bool
where
    D: Ord + Sub<Output = D>,
{
    let diff = if left >= right {
        left - right
    } else {
        right - left
    };

    diff <= epsilon
}
