use std::{collections::HashSet, usize};

use serde_derive::{Deserialize, Serialize};

use super::{
    edge::{DirectedHeadlessWeightedEdge, DirectedTaillessWeightedEdge},
    types::VertexId,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChGraph {
    pub number_of_verticies: u32,
    pub upward_graph: Vec<Vec<DirectedTaillessWeightedEdge>>,
    pub downward_graph: Vec<Vec<DirectedHeadlessWeightedEdge>>,
}

impl ChGraph {
    pub fn upward_neighborhood(&self, vertex: VertexId) -> HashSet<VertexId> {
        self.upward_graph[vertex as usize]
            .iter()
            .map(|edge| edge.head)
            .collect()
    }

    pub fn downward_neighborhood(&self, vertex: VertexId) -> HashSet<VertexId> {
        self.downward_graph[vertex as usize]
            .iter()
            .map(|edge| edge.tail)
            .collect()
    }
}
