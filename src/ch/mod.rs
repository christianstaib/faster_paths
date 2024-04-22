use crate::graphs::{edge::DirectedWeightedEdge, VertexId};

pub mod all_in_preprocessor;
pub mod ch_priority_element;
pub mod contracted_graph;
pub mod contractor;
pub mod preprocessor;
pub mod priority_function;
pub mod shortcut_replacer;

#[derive(Clone)]
pub struct Shortcut {
    pub edge: DirectedWeightedEdge,
    pub vertex: VertexId,
}
