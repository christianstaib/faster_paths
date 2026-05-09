use std::{cmp::Ordering, usize};

use crate::types::{Distance, VertexId};

#[derive(Clone, Copy, Debug)]
pub struct LabelEntry<D> {
    pub hub: VertexId,
    pub weight: D,
    pub predecessor: Option<VertexId>,
}

pub fn min_distance_intersection<D: Distance>(
    dir1_label: &[LabelEntry<D>],
    dir2_label: &[LabelEntry<D>],
) -> Option<(D, usize, usize)> {
    let mut dir1_index = 0;
    let mut dir2_index = 0;
    let mut best = None;

    while dir1_index < dir1_label.len() && dir2_index < dir2_label.len() {
        match dir1_label[dir1_index].hub.cmp(&dir2_label[dir2_index].hub) {
            Ordering::Less => dir1_index += 1,
            Ordering::Greater => dir2_index += 1,
            Ordering::Equal => {
                let distance = dir1_label[dir1_index].weight + dir2_label[dir2_index].weight;

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

pub fn path<D: Distance>(label: &[LabelEntry<D>], index: usize) -> Option<Vec<VertexId>> {
    let mut entry = label.get(index)?;
    let mut path = vec![entry.hub];

    while let Some(predecessor) = entry.predecessor {
        let predecessor_index = label
            .binary_search_by_key(&predecessor, |entry| entry.hub)
            .ok()?;

        entry = &label[predecessor_index];
        path.push(entry.hub);
    }

    Some(path)
}
