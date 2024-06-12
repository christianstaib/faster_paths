use std::usize;

use ahash::HashMap;
use serde::{Deserialize, Serialize};

use super::{label::Label, HubGraphTrait};
use crate::graphs::{edge::DirectedEdge, VertexId};

#[derive(Serialize, Deserialize)]
pub struct DirectedHubGraph {
    pub forward_labels: Vec<Label>,
    pub reverse_labels: Vec<Label>,
    pub shortcuts: HashMap<DirectedEdge, VertexId>,
}

impl HubGraphTrait for DirectedHubGraph {
    fn forward_label(&self, vertex: VertexId) -> Option<&Label> {
        self.forward_labels.get(vertex as usize)
    }

    fn reverse_label(&self, vertex: VertexId) -> Option<&Label> {
        self.reverse_labels.get(vertex as usize)
    }
}
