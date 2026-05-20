use crate::{
    contraction_hierachy::{ContractionEdge, ContractionHierarchy, extract_contraction_order},
    flattened_nested::FlattenedNested,
    graph::{CsrGraph, GraphLike},
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
    let top_down_order = extract_contraction_order(ch).unwrap();

    let num_vertices = ch.num_vertices();
    let mut up_labels = initialize_labels(num_vertices);
    let mut down_labels = initialize_labels(num_vertices);

    let bar = ProgressBar::new(num_vertices as u64);
    for vertices in top_down_order {
        // Build labels in parallel
        let labels = vertices
            .iter()
            .inspect(|_| bar.inc(1))
            .par_bridge()
            .map(|&vertex| {
                let mut up_label = merge_label(ch.up_graph(), &up_labels, vertex);
                up_label = prune_label(&down_labels, &up_label);
                up_label.shrink_to_fit();

                let mut down_label = merge_label(ch.down_graph(), &down_labels, vertex);
                down_label = prune_label(&up_labels, &down_label);
                down_label.shrink_to_fit();

                (vertex, up_label, down_label)
            })
            .collect::<Vec<_>>();

        // Assign them sequential. Could potentially use unsafe here.
        labels
            .into_iter()
            .for_each(|(vertex, up_label, down_label)| {
                up_labels[vertex.as_usize()] = up_label;
                down_labels[vertex.as_usize()] = down_label;
            });

        // bar.inc(vertices.len() as u64);
    }
    bar.finish();

    let up_hub_labeling = FlattenedNested::new(&up_labels);
    let down_hub_labeling = FlattenedNested::new(&down_labels);
    HubLabeling {
        up_hub_labeling,
        down_hub_labeling,
    }
}

fn initialize_labels<D: Distance>(num_vertices: usize) -> Vec<Vec<LabelEntry<D>>> {
    (0..num_vertices)
        .map(|vertex| {
            vec![LabelEntry {
                hub: VertexId::new(vertex as u32),
                distance: D::zero(),
                predecessor_hub: None,
            }]
        })
        .collect()
}

fn merge_label<D: Distance>(
    dir1_graph: &CsrGraph<ContractionEdge<D>>,
    dir1_labels: &[Vec<LabelEntry<D>>],
    vertex: VertexId,
) -> Vec<LabelEntry<D>> {
    let mut new_label: FxHashMap<VertexId, LabelEntry<D>> = FxHashMap::default();
    new_label.insert(
        vertex,
        LabelEntry {
            hub: vertex,
            distance: D::zero(),
            predecessor_hub: None,
        },
    );

    for edge in dir1_graph.outgoing_edges(vertex) {
        for entry in dir1_labels[edge.head.as_usize()].iter() {
            let new_entry = LabelEntry {
                hub: entry.hub,
                distance: entry.distance + edge.weight,
                predecessor_hub: Some(entry.predecessor_hub.unwrap_or(vertex)),
            };
            if let Some(old_entry) = new_label.get_mut(&entry.hub) {
                if new_entry.distance < old_entry.distance {
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

/// Returns a pruned `dir1_label` by removing all entries whose distance is not equal to the true
/// distance, as they can never contribute to a true shortest-distance query.
fn prune_label<D: Distance>(
    dir2_labels: &[Vec<LabelEntry<D>>],
    dir1_label: &[LabelEntry<D>],
) -> Vec<LabelEntry<D>> {
    dir1_label
        .iter()
        .filter(|entry| {
            let dir2_label = &dir2_labels[entry.hub.as_usize()];
            let (true_distance, _dir1_index, _dir2_index) =
                min_distance_intersection(&dir1_label, dir2_label).unwrap();

            entry.distance == true_distance
        })
        .copied()
        .collect()
}
