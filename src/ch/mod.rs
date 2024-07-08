use serde::{Deserialize, Serialize};

use crate::graphs::{edge::DirectedWeightedEdge, VertexId};

pub mod ch_from_top_down;
pub mod ch_priority_element;
pub mod contraction_adaptive_non_simulated;
pub mod contraction_adaptive_simulated;
pub mod contraction_non_adaptive;
pub mod contractor;
pub mod directed_contracted_graph;
pub mod helpers;
pub mod pathfinding;
pub mod priority_function;

#[derive(Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub edge: DirectedWeightedEdge,
    pub vertex: VertexId,
}
