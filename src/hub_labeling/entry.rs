use std::cmp::Ordering;

use crate::types::{Distance, VertexId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LabelEntry<D> {
    pub hub: VertexId,
    pub distance: D,
    pub predecessor_hub: Option<VertexId>,
}

/// Finds the common hub with the smallest combined distance.
///
/// Both labels must be sorted by hub. Returns the best distance together with
/// the matching indices in `dir1_label` and `dir2_label`.
pub fn min_common_hub_distance<D: Distance>(
    dir1_label: &[LabelEntry<D>],
    dir2_label: &[LabelEntry<D>],
) -> Option<(D, usize, usize)> {
    let mut dir1_index = 0;
    let mut dir2_index = 0;
    let mut best = None;

    while dir1_index < dir1_label.len() && dir2_index < dir2_label.len() {
        let dir1_entry = &dir1_label[dir1_index];
        let dir2_entry = &dir2_label[dir2_index];
        match dir1_entry.hub.cmp(&dir2_entry.hub) {
            Ordering::Less => dir1_index += 1,
            Ordering::Greater => dir2_index += 1,
            Ordering::Equal => {
                let distance = dir1_entry.distance + dir2_entry.distance;

                if best.is_none_or(|(best_distance, _, _)| distance < best_distance) {
                    best = Some((distance, dir1_index, dir2_index));
                }

                dir1_index += 1;
                dir2_index += 1;
            }
        }
    }

    best
}

/// Builds a reversed shortcut path from the label entry given by index to its root.
///
/// The label must be sorted by hub. Starting at `label[index]`, the path follows
/// predecessor hubs until it reaches an entry without a predecessor.
pub fn reversed_shortcut_path<D: Distance>(
    label: &[LabelEntry<D>],
    index: usize,
) -> Option<Vec<VertexId>> {
    let mut entry = label.get(index)?;
    let mut path = vec![entry.hub];

    while let Some(predecessor_hub) = entry.predecessor_hub {
        let predecessor_index = label
            .binary_search_by_key(&predecessor_hub, |entry| entry.hub)
            .ok()?;

        entry = &label[predecessor_index];
        path.push(entry.hub);
    }

    Some(path)
}
