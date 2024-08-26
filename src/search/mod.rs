use crate::graphs::{Distance, Vertex};

pub mod alt;
pub mod ch;
pub mod collections;
pub mod dijkstra;
pub mod hl;
pub mod path;
pub mod shortcuts;

pub trait DistanceHeuristic: Send + Sync {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Option<Distance>;

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Option<Distance>;

    fn is_less_or_equal_upper_bound(
        &self,
        source: Vertex,
        target: Vertex,
        distance: Distance,
    ) -> bool;
}
