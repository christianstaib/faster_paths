use crate::graphs::{edge::DirectedWeightedEdge, types::VertexId};

pub mod ch_queue;
pub mod contraction_helper;
pub mod contractor;
pub mod preprocessor;
pub mod shortcut_replacer;

#[derive(Clone)]
pub struct Shortcut {
    pub edge: DirectedWeightedEdge,
    pub vertex: VertexId,
}
