use crate::{
    path::{Path, PathQuery},
    types::Distance,
};

/// Main trait of the crate.
/// Provides an interface that muliple algorithms can serve.
pub trait ShortestPathFinder {
    type Distance: Distance;

    /// Returns the path from `source` to `target`.
    fn path(&mut self, query: &PathQuery) -> Option<Path<Self::Distance>>;

    /// Returns the shortest path distance from `source` to `target`.
    fn distance(&mut self, query: &PathQuery) -> Option<Self::Distance>;
}
