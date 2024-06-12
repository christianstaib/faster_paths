use super::{directed_hub_graph::DirectedHubGraph, label::Label, HubGraphTrait};
use crate::{
    graphs::{
        path::{Path, PathFinding, ShortestPathRequest},
        Weight,
    },
    shortcut_replacer::slow_shortcut_replacer::replace_shortcuts_slow,
};

impl PathFinding for DirectedHubGraph {
    fn shortest_path(&self, path_request: &ShortestPathRequest) -> Option<Path> {
        // wanted: source -> target
        let forward_label = self.forward_label(path_request.source())?;
        let backward_label = self.reverse_label(path_request.target())?;

        let mut path = shortest_path(forward_label, backward_label);

        if let Some(path) = path.as_mut() {
            replace_shortcuts_slow(&mut path.vertices, &self.shortcuts);
        }

        path
    }

    fn shortest_path_weight(&self, path_request: &ShortestPathRequest) -> Option<Weight> {
        let forward_label = self.forward_label(path_request.source())?;
        let backward_label = self.reverse_label(path_request.target())?;

        shortest_path_weight(forward_label, backward_label)
    }

    fn number_of_vertices(&self) -> u32 {
        self.forward_labels.len() as u32
    }
}

pub fn shortest_path(forward_label: &Label, backward_label: &Label) -> Option<Path> {
    // wanted: source -> target
    let (_, forward_index, reverse_index) = overlap(forward_label, backward_label)?;

    // unwrap can be called as be found overlapp and therefore path exists
    let mut forward_path = forward_label.get_path(forward_index).unwrap();
    let reverse_path = backward_label.get_path(reverse_index).unwrap();

    // now got: forward_path (meeting -> source) and reverse_reverse (meeting ->
    // target)
    forward_path.vertices.reverse();
    forward_path.vertices.pop();

    forward_path.vertices.extend(reverse_path.vertices);
    forward_path.weight += reverse_path.weight;

    Some(forward_path)
}

pub fn shortest_path_weight(forward_label: &Label, backward_label: &Label) -> Option<Weight> {
    let (weight, _, _) = overlap(forward_label, backward_label)?;

    Some(weight)
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
