use crate::{
    path::{Path, Query},
    types::Distance,
};

/// Common interface for shortest-path query engines.
///
/// Methods take `&mut self` so that pathfinders can reuse their internal data
/// structures for better performance.
pub trait ShortestPathFinder {
    type Distance: Distance;

    /// Returns a shortest path from `query.source` to `query.target`, or `None` if none exists.
    ///
    /// Returns *a* shortest path, since the shortest path is not necessarily unique.
    fn path(&mut self, query: &Query) -> Option<Path<Self::Distance>>;

    /// Returns the shortest path distance from `query.source` to `query.target`, or `None` if
    /// there is no path between them.
    fn distance(&mut self, query: &Query) -> Option<Self::Distance>;
}
