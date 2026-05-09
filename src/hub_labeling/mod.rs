pub mod entry;

use std::{
    cmp::Reverse,
    collections::{BinaryHeap, VecDeque},
};

use crate::{
    contraction_hierachy::{ContractionEdge, ContractionHierarchy},
    flattened_nested::FlattenedNested,
    graph::{FastGraph, GraphLike},
    types::{Distance, VertexId},
};

use entry::{LabelEntry, min_distance_intersection};
use indicatif::ProgressBar;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

pub struct HubLabeling<D: Distance> {
    pub up_hub_labeling: FlattenedNested<LabelEntry<D>>,
    pub down_hub_labeling: FlattenedNested<LabelEntry<D>>,
}

pub fn merge<D: Distance + Send + Sync>(ch: &ContractionHierarchy<D>) -> HubLabeling<D> {
    let top_down_order = top_down_order(ch);

    let num_vertices = ch.num_vertices();
    let mut up_labels = self_labels(num_vertices);
    let mut down_labels = self_labels(num_vertices);

    let bar = ProgressBar::new(num_vertices as u64);
    for vertices in top_down_order {
        for vertex in vertices {
            merge_label(ch.up_graph(), vertex, &mut up_labels, &down_labels);
            merge_label(ch.down_graph(), vertex, &mut down_labels, &up_labels);
            bar.inc(1);
        }
    }
    bar.finish();

    HubLabeling {
        up_hub_labeling: FlattenedNested::new(&up_labels),
        down_hub_labeling: FlattenedNested::new(&down_labels),
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

fn merge_label<D: Distance + Send + Sync>(
    dir1_graph: &FastGraph<ContractionEdge<D>>,
    vertex: VertexId,
    dir1_labels: &mut [Vec<LabelEntry<D>>],
    dir2_labels: &[Vec<LabelEntry<D>>],
) {
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
            if let Some(old_entry) = new_label.get_mut(&edge.head) {
                if new_entry.weight < old_entry.weight {
                    *old_entry = new_entry;
                }
            } else {
                new_label.insert(edge.head, new_entry);
            }
        }
    }

    let mut new_label: Vec<LabelEntry<D>> = new_label.into_values().collect();

    prune_label(&mut new_label, dir2_labels);
    dir1_labels[vertex.as_usize()] = new_label;
}

fn prune_label<D: Distance + Send + Sync>(
    dir1_label: &mut Vec<LabelEntry<D>>,
    dir2_labels: &[Vec<LabelEntry<D>>],
) {
    let old_dir1_label = dir1_label.clone();
    *dir1_label = old_dir1_label
        .iter()
        .copied()
        .filter(|entry| {
            let true_distance =
                min_distance_intersection(&old_dir1_label, &dir2_labels[entry.hub.as_usize()])
                    .unwrap()
                    .0;

            entry.weight == true_distance
        })
        .collect();
}

fn top_down_order<D: Distance>(ch: &ContractionHierarchy<D>) -> Vec<Vec<VertexId>> {
    let num_vertices = ch.num_vertices();
    let mut indegrees = vec![0; num_vertices];

    for edge in ch.up_graph().edges().chain(ch.down_graph().edges()) {
        indegrees[edge.head.as_usize()] += 1;
    }

    let mut current_layer = indegrees
        .iter()
        .enumerate()
        .filter_map(|(vertex, &indegree)| (indegree == 0).then_some(VertexId::new(vertex as u32)))
        .collect::<VecDeque<_>>();
    let mut layers = Vec::new();
    let mut visited = 0;

    while !current_layer.is_empty() {
        let mut layer = Vec::with_capacity(current_layer.len());
        let mut next_layer = VecDeque::new();

        while let Some(vertex) = current_layer.pop_front() {
            layer.push(vertex);
            visited += 1;

            for edge in ch
                .up_graph()
                .out_edges(vertex)
                .iter()
                .chain(ch.down_graph().out_edges(vertex))
            {
                let head_indegree = &mut indegrees[edge.head.as_usize()];
                *head_indegree -= 1;

                if *head_indegree == 0 {
                    next_layer.push_back(edge.head);
                }
            }
        }

        layers.push(layer);
        current_layer = next_layer;
    }

    assert_eq!(
        visited, num_vertices,
        "contraction hierarchy must be acyclic"
    );

    layers.reverse();
    layers
}
