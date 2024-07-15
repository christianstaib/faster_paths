use std::{cmp::max, usize};

use ahash::HashMap;
use serde::{Deserialize, Serialize};

use super::{label::LabelEntry, HubGraphTrait};
use crate::graphs::{edge::DirectedEdge, VertexId};

#[derive(Serialize, Deserialize)]
pub struct DirectedHubGraph {
    forward_labels: Vec<LabelEntry>,
    forward_indices: Vec<u32>,

    reverse_labels: Vec<LabelEntry>,
    reverse_indices: Vec<u32>,

    shortcuts: HashMap<DirectedEdge, VertexId>,
}

impl DirectedHubGraph {
    pub fn new(
        forward_labels: Vec<Vec<LabelEntry>>,
        reverse_labels: Vec<Vec<LabelEntry>>,
        shortcuts: HashMap<DirectedEdge, VertexId>,
    ) -> DirectedHubGraph {
        let mut forward_indices = vec![0];
        let mut flattened_forward_labels = Vec::new();
        for label in forward_labels {
            forward_indices.push(forward_indices.last().unwrap() + label.len() as u32);
            flattened_forward_labels.extend(label);
        }

        let mut reverse_indices = vec![0];
        let mut flattened_reverse_labels = Vec::new();
        for label in reverse_labels {
            reverse_indices.push(reverse_indices.last().unwrap() + label.len() as u32);
            flattened_reverse_labels.extend(label);
        }

        DirectedHubGraph {
            forward_labels: flattened_forward_labels,
            forward_indices,
            reverse_labels: flattened_reverse_labels,
            reverse_indices,
            shortcuts,
        }
    }

    pub fn number_of_vertices(&self) -> u32 {
        max(
            self.forward_indices.len() as u32 - 1,
            self.reverse_indices.len() as u32 - 1,
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

    fn reverse_label(&self, vertex: VertexId) -> &[LabelEntry] {
        let start_index = self.reverse_indices[vertex as usize] as usize;
        let end_index = self.reverse_indices[vertex as usize + 1] as usize;
        &self.reverse_labels[start_index..end_index]
    }
}
