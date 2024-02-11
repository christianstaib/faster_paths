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
    pub fn get_avg_label_size(&self) -> f32 {
        let summed_label_size: u64 = self
            .forward_labels
            .iter()
            .map(|label| label.entries.len() as u64)
            .sum::<u64>()
            + self
                .reverse_labels
                .iter()
                .map(|label| label.entries.len() as u64)
                .sum::<u64>();
        summed_label_size as f32 / (2 * self.forward_labels.len()) as f32
    }

    pub fn get_weight(&self, request: &PathRequest) -> Option<u32> {
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.reverse_labels.get(request.target as usize)?;

        Self::get_weight_labels(forward_label, backward_label)
    }

    pub fn get_path(&self, request: &PathRequest) -> Option<Path> {
        let forward_label = self.forward_labels.get(request.source as usize)?;
        let backward_label = self.reverse_labels.get(request.target as usize)?;
        let path_with_shortcuts = Self::get_path_with_shortcuts(forward_label, backward_label)?;

        Some(self.shortcut_replacer.get_route(&path_with_shortcuts))
    }

    // cost, route_with_shortcuts
    pub fn get_path_with_shortcuts(forward: &Label, reverse: &Label) -> Option<Path> {
        let (_, forward_idx, reverse_idx) = Self::get_overlap(forward, reverse)?;
        let mut forward_path = forward.get_path(forward_idx);
        let reverse_path = reverse.get_path(reverse_idx);

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
    pub fn get_overlap(forward: &Label, reverse: &Label) -> Option<(Weight, u32, u32)> {
        let mut i_forward = 0;
        let mut i_reverse = 0;

        let mut overlap_weight = None;
        let mut overlap_i_forward = 0;
        let mut overlap_i_reverse = 0;

        while i_forward < forward.entries.len() && i_reverse < reverse.entries.len() {
            let forward_entry = &forward.entries[i_forward];
            let reverse_entry = &reverse.entries[i_reverse];

            match forward_entry.vertex.cmp(&reverse_entry.vertex) {
                std::cmp::Ordering::Less => i_forward += 1,
                std::cmp::Ordering::Equal => {
                    let alternative_weight = forward_entry
                        .weight
                        .checked_add(reverse_entry.weight)
                        .unwrap();
                    if alternative_weight < overlap_weight.unwrap_or(u32::MAX) {
                        overlap_weight = Some(alternative_weight);
                        overlap_i_forward = i_forward;
                        overlap_i_reverse = i_reverse;
                    }

                    i_forward += 1;
                    i_reverse += 1;
                }
                std::cmp::Ordering::Greater => i_reverse += 1,
            }
        }

        if let Some(min_weight) = overlap_weight {
            return Some((
                min_weight,
                overlap_i_forward as u32,
                overlap_i_reverse as u32,
            ));
        }

        None
    }
}
