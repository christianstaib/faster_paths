use serde::{Deserialize, Serialize};

use crate::graphs::{edge::WeightedEdge, VertexId};

pub mod ch_from_top_down;
pub mod ch_priority_element;
pub mod contraction_adaptive_non_simulated;
pub mod contraction_adaptive_simulated;
pub mod contraction_with_fixed_order;
pub mod contractor;
pub mod directed_contracted_graph;
pub mod helpers;
pub mod pathfinding;
pub mod priority_function;

#[derive(Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub edge: WeightedEdge,
    pub vertex: VertexId,
}
