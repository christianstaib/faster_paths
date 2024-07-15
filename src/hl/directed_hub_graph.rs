use std::{cmp::max, usize};

use ahash::HashMap;
use serde::{Deserialize, Serialize};

use super::{label::LabelEntry, HubGraphTrait};
use crate::graphs::{edge::DirectedEdge, VertexId};

#[derive(Serialize, Deserialize)]
pub struct DirectedHubGraph {
    /// The forward label of vertex `v` is stored in `forward_labels[forward_indices[v]..forward_indices[v + 1]]`
    forward_labels: Vec<LabelEntry>,
    forward_indices: Vec<u32>,

    /// The backward label of vertex `v` is stored in `backward_labels[backward_indices[v]..backward_indices[v + 1]]`
    backward_labels: Vec<LabelEntry>,
    backward_indices: Vec<u32>,

    // Map of shortcuts represented by directed edges and their corresponding vertex IDs
    shortcuts: HashMap<DirectedEdge, VertexId>,
}

impl DirectedHubGraph {
    pub fn new(
        forward_labels: Vec<Vec<LabelEntry>>,
        backward_labels: Vec<Vec<LabelEntry>>,
        shortcuts: HashMap<DirectedEdge, VertexId>,
    ) -> DirectedHubGraph {
        let mut forward_indices = vec![0];
        let mut flattened_forward_labels = Vec::new();
        for label in forward_labels {
            forward_indices.push(forward_indices.last().unwrap() + label.len() as u32);
            flattened_forward_labels.extend(label);
        }

        let mut backward_indices = vec![0];
        let mut flattened_backward_labels = Vec::new();
        for label in backward_labels {
            backward_indices.push(backward_indices.last().unwrap() + label.len() as u32);
            flattened_backward_labels.extend(label);
        }

        DirectedHubGraph {
            forward_labels: flattened_forward_labels,
            forward_indices,
            backward_labels: flattened_backward_labels,
            backward_indices,
            shortcuts,
        }
    }

    pub fn number_of_vertices(&self) -> u32 {
        max(
            self.forward_indices.len() as u32 - 1,
            self.backward_indices.len() as u32 - 1,
        )
    }

    pub fn shortcuts(&self) -> &HashMap<DirectedEdge, VertexId> {
        &self.shortcuts
    }
}

impl HubGraphTrait for DirectedHubGraph {
    fn forward_label(&self, vertex: VertexId) -> &[LabelEntry] {
        let start_index = self.forward_indices[vertex as usize] as usize;
        let end_index = self.forward_indices[vertex as usize + 1] as usize;
        &self.forward_labels[start_index..end_index]
    }

    fn backward_label(&self, vertex: VertexId) -> &[LabelEntry] {
        let start_index = self.backward_indices[vertex as usize] as usize;
        let end_index = self.backward_indices[vertex as usize + 1] as usize;
        &self.backward_labels[start_index..end_index]
    }
}
