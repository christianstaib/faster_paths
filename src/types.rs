use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Sub},
};

use num_traits::{Bounded, Zero};

pub type Vertex = u32;

/// Edge weight and path distance type used by shortest-path algorithms.
///
/// The type must be ordered, copyable, thread-safe, have zero and maximum
/// values, and support addition, in-place addition, and subtraction.
///
/// `Distance` is implemented automatically for every type that satisfies these
/// bounds, so your custom distance types only need to implement the required
/// standard and `num_traits` traits.
///
/// Supported distance types include integer types and floating-point types
/// wrapped in `ordered_float::OrderedFloat`, as native floating-point values do
/// not implement `Ord`.
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
pub(crate) fn abs_diff_eq<D>(left: D, right: D, epsilon: D) -> bool
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
