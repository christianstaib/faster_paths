use collections::dijkstra_data::Path;

use crate::graphs::{Distance, Vertex};

pub mod alt;
pub mod ch;
pub mod collections;
pub mod dijkstra;
pub mod hl;
pub mod path;
pub mod shortcuts;

pub trait DistanceHeuristic: Send + Sync {
    fn lower_bound(&self, _source: Vertex, _target: Vertex) -> Distance {
        0
    }

    fn upper_bound(&self, _source: Vertex, _target: Vertex) -> Distance {
        Distance::MAX
    }

    fn is_less_or_equal_upper_bound(
        &self,
        source: Vertex,
        target: Vertex,
        distance: Distance,
    ) -> bool {
        distance <= self.upper_bound(source, target)
    }
}

pub trait PathFinding: Send + Sync {
    fn shortest_path(&self, source: Vertex, target: Vertex) -> Option<Path>;

    fn shortest_path_distance(&self, source: Vertex, target: Vertex) -> Option<Distance>;
}

pub struct TrivialHeuristic {}

impl DistanceHeuristic for TrivialHeuristic {}

pub struct PathfinderHeuristic<'a> {
    pub pathfinder: &'a dyn PathFinding,
}

impl<'a> DistanceHeuristic for PathfinderHeuristic<'a> {
    fn lower_bound(&self, source: Vertex, target: Vertex) -> Distance {
        self.pathfinder
            .shortest_path_distance(source, target)
            .unwrap_or(0)
    }

    fn upper_bound(&self, source: Vertex, target: Vertex) -> Distance {
        self.pathfinder
            .shortest_path_distance(source, target)
            .unwrap_or(Distance::MAX)
    }
}
