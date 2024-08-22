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

    let mut forward_index = 0;
    let mut backward_index = 0;

    let mut forward_vertex = forward_label
        .get(forward_index as usize)
        .map(|entry| &entry.vertex)
        .unwrap_or(&Vertex::MAX);
    let mut backward_vertex = backward_label
        .get(backward_index as usize)
        .map(|entry| &entry.vertex)
        .unwrap_or(&Vertex::MAX);
    while forward_index < forward_label.len() || backward_index < backward_label.len() {
        match forward_vertex.cmp(&backward_vertex) {
            Ordering::Less => {
                forward_index += 1;
                forward_vertex = forward_label
                    .get(forward_index as usize)
                    .map(|entry| &entry.vertex)
                    .unwrap_or(&Vertex::MAX);
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

                forward_index += 1;
                backward_index += 1;
                forward_vertex = forward_label
                    .get(forward_index as usize)
                    .map(|entry| &entry.vertex)
                    .unwrap_or(&Vertex::MAX);
                backward_vertex = backward_label
                    .get(backward_index as usize)
                    .map(|entry| &entry.vertex)
                    .unwrap_or(&Vertex::MAX);
            }
            Ordering::Greater => {
                backward_index += 1;
                backward_vertex = backward_label
                    .get(backward_index as usize)
                    .map(|entry| &entry.vertex)
                    .unwrap_or(&Vertex::MAX);
            }
        }
    }

    overlapp
}
