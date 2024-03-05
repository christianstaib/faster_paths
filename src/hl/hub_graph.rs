use serde_derive::{Deserialize, Serialize};

use crate::{
    ch::fast_shortcut_replacer::FastShortcutReplacer,
    graphs::path::{Path, ShortestPathRequest},
    graphs::types::Weight,
};

use super::label::Label;

#[derive(Serialize, Deserialize)]
pub struct HubGraph {
    pub forward_labels: Vec<Label>,
    pub reverse_labels: Vec<Label>,
    pub shortcut_replacer: FastShortcutReplacer,
}

impl HubGraph {
    pub fn get_weight(&self, request: &ShortestPathRequest) -> Option<Weight> {
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.reverse_labels.get(request.target as usize)?;
        let (weight, _, _) = Self::overlap(forward_label, backward_label)?;

        Some(weight)
    }

    pub fn get_path(&self, request: &ShortestPathRequest) -> Option<Path> {
        // wanted: source -> target
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.reverse_labels.get(request.target as usize)?;

        let (_, forward_index, reverse_index) = Self::overlap(forward_label, backward_label)?;
        let mut forward_path = forward_label.get_path(forward_index)?;
        let reverse_path = backward_label.get_path(reverse_index)?;

        // now got: forward(meeting -> source) and reverse (meeting -> target)
        forward_path.vertices.reverse();
        forward_path.vertices.pop();

        forward_path.vertices.extend(reverse_path.vertices);
        forward_path.weight += reverse_path.weight;

        let path = self.shortcut_replacer.get_path(&forward_path);

        Some(path)
    }

    /// Calculates the minimal overlap between a forward and reverse label.
    ///
    /// If there exists an overlap, `Some(weight, index_forward, index_reverse)` is returned, where
    /// `weight` is the weight of the shortest path from the source vertex represented by the forward label to the target
    /// vertex represented by the reverse label. `index_forward` is the index of the entry that
    /// represents the shortest path from the source to the meeting vertex, and `index_reverse` is the index of the
    /// entry that represents the shortest path from the meeting vertex to the target.
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
                        .or_else(|| {
                            Some((combined_weight, index_forward as u32, index_reverse as u32))
                        });

                    index_forward += 1;
                    index_reverse += 1;
                }
                std::cmp::Ordering::Greater => index_reverse += 1,
            }
        }

        overlap
    }
}
