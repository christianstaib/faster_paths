use crate::{
    contraction_hierachy::{ContractionEdge, ContractionHierarchy, extract_contraction_order},
    flattened_nested::FlattenedNested,
    graph::{FastGraph, GraphLike},
    hub_labeling::{
        HubLabeling,
        entry::{LabelEntry, min_distance_intersection},
    },
    types::{Distance, VertexId},
};

use rayon::prelude::*;

use indicatif::ProgressBar;
use rustc_hash::FxHashMap;

pub fn merge<D: Distance + Send + Sync>(ch: &ContractionHierarchy<D>) -> HubLabeling<D> {
    let top_down_order = extract_contraction_order(ch);

    let num_vertices = ch.num_vertices();
    let mut up_labels = self_labels(num_vertices);
    let mut down_labels = self_labels(num_vertices);

    let bar = ProgressBar::new(num_vertices as u64);
    for vertices in top_down_order {
        // Build labels in parallel
        let labels = vertices
            .par_iter()
            .map(|&vertex| {
                let mut up_label = merge_label(ch.up_graph(), vertex, &up_labels);
                up_label = prune_label(&up_label, &down_labels);
                up_label.shrink_to_fit();
                let mut down_label = merge_label(ch.down_graph(), vertex, &down_labels);
                down_label = prune_label(&down_label, &up_labels);
                down_label.shrink_to_fit();
                (vertex, up_label, down_label)
            })
            .collect::<Vec<_>>();

        // Assign them sequential
        labels
            .into_iter()
            .for_each(|(vertex, up_label, down_label)| {
                up_labels[vertex.as_usize()] = up_label;
                down_labels[vertex.as_usize()] = down_label;
            });

        bar.inc(vertices.len() as u64);
    }
    bar.finish();

    let up_hub_labeling = FlattenedNested::new(&up_labels);
    let down_hub_labeling = FlattenedNested::new(&down_labels);
    HubLabeling {
        up_hub_labeling,
        down_hub_labeling,
    }
}

fn self_labels<D: Distance>(num_vertices: usize) -> Vec<Vec<LabelEntry<D>>> {
    (0..num_vertices)
        .map(|vertex| {
            vec![LabelEntry {
                hub: VertexId::new(vertex as u32),
                weight: D::zero(),
                predecessor: None,
            }]
        })
        .collect()
}

fn merge_label<D: Distance>(
    dir1_graph: &FastGraph<ContractionEdge<D>>,
    vertex: VertexId,
    dir1_labels: &[Vec<LabelEntry<D>>],
) -> Vec<LabelEntry<D>> {
    let mut new_label: FxHashMap<VertexId, LabelEntry<D>> = FxHashMap::default();
    new_label.insert(
        vertex,
        LabelEntry {
            hub: vertex,
            weight: D::zero(),
            predecessor: None,
        },
    );

    for edge in dir1_graph.out_edges(vertex) {
        for entry in dir1_labels[edge.head.as_usize()].iter() {
            let new_entry = LabelEntry {
                hub: entry.hub,
                weight: entry.weight + edge.weight,
                predecessor: Some(entry.predecessor.unwrap_or(vertex)),
            };
            if let Some(old_entry) = new_label.get_mut(&entry.hub) {
                if new_entry.weight < old_entry.weight {
                    *old_entry = new_entry;
                }
            } else {
                new_label.insert(entry.hub, new_entry);
            }
        }
    }

    let mut new_label: Vec<LabelEntry<D>> = new_label.into_values().collect();
    new_label.sort_unstable_by_key(|entry| entry.hub);
    new_label
}

fn prune_label<D: Distance>(
    dir1_label: &[LabelEntry<D>],
    dir2_labels: &[Vec<LabelEntry<D>>],
) -> Vec<LabelEntry<D>> {
    dir1_label
        .iter()
        .filter(|entry| {
            let true_distance =
                min_distance_intersection(&dir1_label, &dir2_labels[entry.hub.as_usize()])
                    .unwrap()
                    .0;

            entry.weight == true_distance
        })
        .copied()
        .collect()
}
