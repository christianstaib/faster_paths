use ahash::{HashMap, HashMapExt};
use indicatif::ProgressBar;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::{
    directed_hub_graph::DirectedHubGraph,
    label::{new_label, LabelEntry},
    pathfinding::overlap,
};
use crate::{
    ch::directed_contracted_graph::DirectedContractedGraph,
    graphs::{Graph, VertexId},
};

pub fn directed_hub_graph_from_directed_contracted_graph(
    ch_information: &DirectedContractedGraph,
) -> DirectedHubGraph {
    let mut forward_labels: Vec<_> = (0..ch_information.upward_graph.number_of_vertices())
        .map(new_label)
        .collect();

    let mut reverse_labels = forward_labels.clone();

    let pb = ProgressBar::new(ch_information.upward_graph.number_of_vertices() as u64);
    for level_list in ch_information.level_to_vertices_map.iter().rev() {
        let labels: Vec<_> = level_list
            .par_iter()
            .map(|&vertex| {
                let forward_label = generate_forward_label(
                    ch_information,
                    vertex,
                    &forward_labels,
                    &reverse_labels,
                );
                let reverse_label = generate_reverse_label(
                    ch_information,
                    vertex,
                    &forward_labels,
                    &reverse_labels,
                );

                (vertex, forward_label, reverse_label)
            })
            .collect();
        for (vertex, forward_label, reverse_label) in labels {
            forward_labels[vertex as usize] = forward_label;
            reverse_labels[vertex as usize] = reverse_label;
        }
        pb.inc(level_list.len() as u64);
    }
    pb.finish();

    // Needs to be called after all labels are creates as replacing the predecessor
    // VertexId with the index of predecessor in label makes merging impossible.
    forward_labels
        .iter_mut()
        .chain(reverse_labels.iter_mut())
        .for_each(set_predecessor);

    let shortcuts = ch_information.shortcuts.clone();

    DirectedHubGraph::new(forward_labels, reverse_labels, shortcuts)
}

/// Generates a forward label for a given vertex.
///
/// This function constructs a forward label for the specified vertex by
/// considering outgoing edges. It accumulates the weights and predecessors from
/// the labels associated with the direction towards the destination (forward)
/// and adjusts them based on the graph's structure. The resulting label is
/// optimized by merging similar paths and pruning paths that are not efficient
/// when considered with the reverse direction paths.
fn generate_forward_label(
    ch_information: &DirectedContractedGraph,
    vertex: VertexId,
    forward_labels: &Vec<Vec<LabelEntry>>,
    reverse_labels: &Vec<Vec<LabelEntry>>,
) -> Vec<LabelEntry> {
    let mut labels = Vec::new();
    for out_edge in ch_information.upward_graph.out_edges(vertex) {
        let mut label = forward_labels[out_edge.head() as usize].clone();
        label.iter_mut().for_each(|entry| {
            if entry.predecessor_index == u32::MAX {
                entry.predecessor_index = vertex;
            }
            entry.weight += out_edge.weight();
        });
        labels.push(label);
    }
    let mut label = merge(labels, vertex);
    prune(&mut label, reverse_labels);
    label
}

/// Generates a reverse label for a given vertex.
///
/// This function constructs a reverse label for the specified vertex by
/// considering incoming edges. It accumulates the weights and predecessors from
/// the labels associated with the direction from the source (reverse) and
/// adjusts them based on the graph's structure. The resulting label is
/// optimized by merging similar paths and pruning paths that are not efficient
/// when considered with the forward direction paths.
fn generate_reverse_label(
    ch_information: &DirectedContractedGraph,
    vertex: VertexId,
    forward_labels: &Vec<Vec<LabelEntry>>,
    reverse_labels: &Vec<Vec<LabelEntry>>,
) -> Vec<LabelEntry> {
    let mut labels = Vec::new();
    for in_edge in ch_information.downward_graph.out_edges(vertex) {
        let mut label = reverse_labels[in_edge.head() as usize].clone();
        label.iter_mut().for_each(|entry| {
            if entry.predecessor_index == u32::MAX {
                entry.predecessor_index = vertex;
            }
            entry.weight += in_edge.weight();
        });
        labels.push(label);
    }
    let mut label = merge(labels, vertex);
    prune(&mut label, forward_labels);
    label
}

pub fn set_predecessor(label: &mut Vec<LabelEntry>) {
    // maps vertex -> index
    let mut vertex_to_index_map = HashMap::new();
    for idx in 0..label.len() {
        vertex_to_index_map.insert(label[idx].vertex, idx as u32);
    }

    // replace predecessor VertexId with index of predecessor
    for entry in label.iter_mut() {
        if entry.predecessor_index != u32::MAX {
            entry.predecessor_index = *vertex_to_index_map.get(&entry.predecessor_index).unwrap();
        }
    }
}

fn merge(mut labels: Vec<Vec<LabelEntry>>, vertex: VertexId) -> Vec<LabelEntry> {
    // poping from end of vec is faster as poping from beginning
    labels.iter_mut().for_each(|label| label.reverse());

    let mut label_entries = Vec::new();
    labels.push(new_label(vertex));

    while !labels.is_empty() {
        let min_vertex = labels
            .iter()
            .map(|label| label.last().unwrap().vertex)
            .min()
            .unwrap();
        let entries: Vec<_> = labels
            .iter_mut()
            .filter_map(|label| {
                if label.last().unwrap().vertex == min_vertex {
                    return label.pop();
                }
                None
            })
            .collect();
        labels.retain(|label| !label.is_empty());
        let min_entry = entries
            .into_iter()
            .min_by_key(|entry| entry.weight)
            .unwrap();
        label_entries.push(min_entry);
    }

    label_entries
}

fn prune(direction1_label: &mut Vec<LabelEntry>, direction2_labels_labels: &[Vec<LabelEntry>]) {
    let mut new = direction1_label
        .par_iter()
        .filter(|entry| {
            let reverse_label = &direction2_labels_labels[entry.vertex as usize];
            let true_weight = overlap(direction1_label, reverse_label).unwrap().0;
            entry.weight == true_weight
        })
        .cloned()
        .collect::<Vec<_>>();

    std::mem::swap(&mut new, direction1_label);
}
