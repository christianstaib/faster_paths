use crate::{
    ch::ShortcutSearchResult,
    graphs::{graph::Graph, types::VertexId},
};

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
