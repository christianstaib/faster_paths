use serde::{Deserialize, Serialize};

use crate::graphs::{
    edge::{DirectedEdge, DirectedWeightedEdge},
    graph::Graph,
    types::VertexId,
};

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

#[derive(Clone, Serialize, Deserialize)]
pub struct ContractedGraph {
    pub graph: Graph,
    pub shortcuts: Vec<(DirectedEdge, VertexId)>,
    pub levels: Vec<Vec<u32>>,
}
