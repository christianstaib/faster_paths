use std::cmp::Ordering;

use crate::graphs::{Distance, Vertex};

pub struct HubGraph {
    pub forward_labels: Vec<HubLabelEntry>,
    pub forward_indices: Vec<(u32, u32)>,
    pub backward_labels: Vec<HubLabelEntry>,
    pub backward_indices: Vec<(u32, u32)>,
}

#[derive(Debug)]
pub struct HubLabelEntry {
    pub vertex: Vertex,
    pub distance: Distance,
    /// relative index of predecessor. Zero if no predecessor.
    pub predecessor_index: Option<u32>,
}

pub fn overlapp(
    forward_label: &[HubLabelEntry],
    backward_label: &[HubLabelEntry],
) -> Option<(Distance, (usize, usize))> {
    let mut overlapp = None;

    let mut forward_iter = forward_label
        .iter()
        .enumerate()
        .map(|(index, entry)| (index, entry.vertex))
        .peekable();
    let mut backward_iter = backward_label
        .iter()
        .enumerate()
        .map(|(index, entry)| (index, entry.vertex))
        .peekable();

    while let (Some(&(forward_index, forward_vertex)), Some(&(backward_index, backward_vertex))) =
        (forward_iter.peek(), backward_iter.peek())
    {
        match forward_vertex.cmp(&backward_vertex) {
            Ordering::Less => {
                forward_iter.next();
            }
            Ordering::Equal => {
                let alternative_distance = forward_label[forward_index as usize].distance
                    + backward_label[backward_index as usize].distance;
                if alternative_distance
                    < overlapp
                        .map(|(current_distance, _)| current_distance)
                        .unwrap_or(Distance::MAX)
                {
                    overlapp = Some((alternative_distance, (forward_index, backward_index)));
                }

                forward_iter.next();
                backward_iter.next();
            }
            Ordering::Greater => {
                backward_iter.next();
            }
        }
    }

    overlapp
}
