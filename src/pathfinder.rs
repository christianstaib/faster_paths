use crate::{
    path::{Path, Query},
    types::Distance,
};

/// Main trait of the crate.
/// Provides an interface that muliple algorithms can serve.
pub trait ShortestPathFinder {
    type Distance: Distance;

    /// Returns the path from `source` to `target`.
    fn path(&mut self, query: &Query) -> Option<Path<Self::Distance>>;

    /// Returns the shortest path distance from `source` to `target`.
    fn distance(&mut self, query: &Query) -> Option<Self::Distance>;
}
