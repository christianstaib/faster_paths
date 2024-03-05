use crate::{
    ch::contraction_helper::ShortcutSearchResult,
    graphs::{graph::Graph, types::VertexId},
};

pub trait PriorityTerm {
    /// Gets the priority of node v in the graph
    fn priority(
        &self,
        vertex: VertexId,
        graph: &Graph,
        shortcuts_results: &ShortcutSearchResult,
    ) -> i32;

    /// Gets called just BERFORE a vertex is contracted. Gives priority terms the oppernunity to updated
    /// neighboring nodes priorities.
    fn update_before_contraction(&mut self, vertex: VertexId, graph: &Graph);
}
