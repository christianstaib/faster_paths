use serde::{Deserialize, Serialize};

use crate::graphs::{
    edge::{DirectedEdge, DirectedWeightedEdge},
    fast_graph::FastGraph,
    types::VertexId,
};

pub mod contractor;
pub mod preprocessor;
pub mod priority_function;
pub mod queue;
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

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractedGraphInformation {
    pub ch_graph: FastGraph,
    pub shortcuts: Vec<(DirectedEdge, VertexId)>,
    pub levels: Vec<Vec<u32>>,
}
