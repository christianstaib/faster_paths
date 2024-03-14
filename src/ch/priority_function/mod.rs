use crate::graphs::{graph::Graph, types::VertexId};

use super::ShortcutSearchResult;

pub mod cost_of_queries;
pub mod deleted_neighbors;
pub mod edge_difference;
pub mod search_space_size;
pub mod voronoi_region;

pub trait PriorityFunction {
    /// Gets the priority of node v in the graph
    fn priority(
        &self,
        vertex: VertexId,
        graph: &Graph,
        shortcuts_results: &ShortcutSearchResult,
    ) -> i32;

    /// Gets called just ERFORE a vertex is contracted. Gives priority terms the oppernunity to updated
    /// neighboring nodes priorities.
    fn update(&mut self, vertex: VertexId, graph: &Graph);
}
