use crate::graphs::{Distance, Vertex};

pub mod alt;
pub mod ch;
pub mod collections;
pub mod dijkstra;
pub mod path;

pub trait DistanceHeuristic {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Option<Distance>;

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Option<Distance>;
}
