use crate::graphs::{edge::DirectedWeightedEdge, types::VertexId};

#[derive(Clone)]
pub struct Shortcut {
    pub edge: DirectedWeightedEdge,
    pub skiped_vertex: VertexId,
}
