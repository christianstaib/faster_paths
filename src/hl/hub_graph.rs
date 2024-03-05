use serde_derive::{Deserialize, Serialize};

use crate::{
    ch::fast_shortcut_replacer::FastShortcutReplacer,
    graphs::path::{Path, PathRequest},
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
    pub fn get_weight(&self, request: &PathRequest) -> Option<Weight> {
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.reverse_labels.get(request.target as usize)?;

        Self::get_weight_labels(forward_label, backward_label)
    }

    pub fn get_path(&self, request: &PathRequest) -> Option<Path> {
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.reverse_labels.get(request.target as usize)?;
        let path_with_shortcuts = Self::get_path_with_shortcuts(forward_label, backward_label)?;
        let path = self.shortcut_replacer.get_path(&path_with_shortcuts);

        Some(path)
    }

    // cost, route_with_shortcuts
    pub fn get_path_with_shortcuts(forward: &Label, reverse: &Label) -> Option<Path> {
        let (_, forward_idx, reverse_idx) = Self::get_overlap(forward, reverse)?;
        let mut forward_path = forward.get_path(forward_idx)?;
        let reverse_path = reverse.get_path(reverse_idx)?;

        // wanted: u -> w
        // got: forward v -> u, reverse v -> w
        if forward_path.vertices.first() == reverse_path.vertices.first() {
            forward_path.vertices.remove(0);
        }

        forward_path.vertices.reverse();
        forward_path.vertices.extend(reverse_path.vertices);

        Some(Path {
            vertices: forward_path.vertices,
            weight: forward_path.weight + reverse_path.weight,
        })
    }

    pub fn get_weight_labels(forward: &Label, reverse: &Label) -> Option<Weight> {
        let (weight, _, _) = Self::get_overlap(forward, reverse)?;
        Some(weight)
    }

    /// cost, i_self, i_other
    fn get_overlap(forward: &Label, reverse: &Label) -> Option<(Weight, u32, u32)> {
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
