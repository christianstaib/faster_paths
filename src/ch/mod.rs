use crate::graphs::{edge::DirectedWeightedEdge, types::VertexId};

pub mod ch_queue;
pub mod contractor;
pub mod preprocessor;
pub mod shortcut_replacer;

#[derive(Clone)]
pub struct Shortcut {
    pub edge: DirectedWeightedEdge,
    pub vertex: VertexId,
}

pub struct ShortcutSearchResult {
    pub shortcuts: Vec<Shortcut>,
    pub search_space_size: i32,
    pub edge_difference: i32,
}
