use serde::{Deserialize, Serialize};

use crate::graphs::{edge::DirectedWeightedEdge, VertexId};

pub mod ch_dijkstra;
pub mod ch_priority_element;
pub mod contracted_graph;
pub mod contraction_adaptive_non_simulated;
pub mod contraction_adaptive_simulated;
pub mod contraction_non_adaptive;
pub mod contractor;
pub mod priority_function;

#[derive(Clone, Serialize, Deserialize)]
pub struct Shortcut {
    pub edge: DirectedWeightedEdge,
    pub vertex: VertexId,
}

pub trait ContractedGraphTrait: Send + Sync {
    fn upward_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_>;

    fn downard_edges(
        &self,
        source: VertexId,
    ) -> Box<dyn ExactSizeIterator<Item = DirectedWeightedEdge> + Send + '_>;

    fn number_of_vertices(&self) -> u32;

    fn number_of_edges(&self) -> u32;
}
