use std::usize;

use serde::{Deserialize, Serialize};

use super::{label::Label, HubGraphTrait};
use crate::graphs::{VertexId, Weight};

#[derive(Serialize, Deserialize)]
pub struct DirectedHubGraph {
    pub forward_labels: Vec<Label>,
    pub reverse_labels: Vec<Label>,
}

impl HubGraphTrait for DirectedHubGraph {
    fn forward_label(&self, vertex: VertexId) -> Option<&Label> {
        self.forward_labels.get(vertex as usize)
    }

    fn reverse_label(&self, vertex: VertexId) -> Option<&Label> {
        self.reverse_labels.get(vertex as usize)
    }
}

#[derive(Serialize, Deserialize)]
pub struct HubGraph {
    pub labels: Vec<Label>,
}

impl HubGraphTrait for HubGraph {
    fn forward_label(&self, vertex: VertexId) -> Option<&Label> {
        self.labels.get(vertex as usize)
    }

    fn reverse_label(&self, vertex: VertexId) -> Option<&Label> {
        self.labels.get(vertex as usize)
    }
}

/// Calculates the minimal overlap between a forward and reverse label.
///
/// If there exists an overlap, `Some(weight, index_forward, index_reverse)` is
/// returned, where `weight` is the weight of the shortest path from the source
/// vertex represented by the forward label to the target vertex represented by
/// the reverse label. `index_forward` is the index of the entry that represents
/// the shortest path from the source to the meeting vertex, and `index_reverse`
/// is the index of the entry that represents the shortest path from the meeting
/// vertex to the target.
pub fn overlap(forward: &Label, reverse: &Label) -> Option<(Weight, u32, u32)> {
    let mut index_forward = 0;
    let mut index_reverse = 0;

    let mut overlap = None;

    while index_forward < forward.entries.len() && index_reverse < reverse.entries.len() {
        let forward_entry = &forward.entries.get(index_forward)?;
        let reverse_entry = &reverse.entries.get(index_reverse)?;

        match forward_entry.vertex.cmp(&reverse_entry.vertex) {
            std::cmp::Ordering::Less => index_forward += 1,
            std::cmp::Ordering::Equal => {
                let combined_weight = forward_entry.weight.checked_add(reverse_entry.weight)?;
                overlap = overlap
                    .take()
                    .filter(|&(current_weight, _, _)| current_weight <= combined_weight)
                    .or(Some((
                        combined_weight,
                        index_forward as u32,
                        index_reverse as u32,
                    )));

                index_forward += 1;
                index_reverse += 1;
            }
            std::cmp::Ordering::Greater => index_reverse += 1,
        }
    }

    overlap
}
