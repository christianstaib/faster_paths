use std::cmp::Ordering;

use indicatif::ProgressIterator;
use itertools::Itertools;

use super::half_hub_graph::{get_hub_label_by_merging, HalfHubGraph};
use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Graph, Vertex},
    search::ch::contracted_graph::ContractedGraph,
};

pub struct HubGraph {
    pub forward: HalfHubGraph,
    pub backward: HalfHubGraph,
}

impl HubGraph {
    pub fn by_brute_force<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        vertex_to_level: &Vec<u32>,
    ) -> HubGraph {
        let forward = HalfHubGraph::by_brute_force(graph.out_graph(), vertex_to_level);
        let backward = HalfHubGraph::by_brute_force(graph.in_graph(), vertex_to_level);

        HubGraph { forward, backward }
    }

    pub fn by_merging(graph: &ContractedGraph) -> HubGraph {
        let mut forward_labels = graph
            .upward_graph
            .vertices()
            .map(|vertex| vec![HubLabelEntry::new(vertex)])
            .collect_vec();

        let mut backward_labels = graph
            .downward_graph
            .vertices()
            .map(|vertex| vec![HubLabelEntry::new(vertex)])
            .collect_vec();

        for &vertex in graph.level_to_vertex.iter().rev().progress() {
            create_label(
                &graph.upward_graph,
                vertex,
                &mut forward_labels,
                &backward_labels,
            );
            create_label(
                &graph.downward_graph,
                vertex,
                &mut backward_labels,
                &forward_labels,
            );
        }

        let forward = HalfHubGraph::new(&forward_labels);
        let backward = HalfHubGraph::new(&backward_labels);

        HubGraph { forward, backward }
    }
}

fn create_label(
    contracted_graph_direction1: &dyn Graph,
    vertex: u32,
    labels_direction1: &mut Vec<Vec<HubLabelEntry>>,
    labels_direction2: &Vec<Vec<HubLabelEntry>>,
) {
    let mut neighbor_labels = contracted_graph_direction1
        .edges(vertex)
        .map(|edge| {
            let neighbor_label = labels_direction1.get(edge.head as usize).unwrap();
            (Some(edge.clone()), neighbor_label)
        })
        .collect::<Vec<_>>();
    neighbor_labels.push((None, labels_direction1.get(vertex as usize).unwrap()));

    let mut forward_label = get_hub_label_by_merging(&neighbor_labels);
    prune_label(&mut forward_label, labels_direction2);
    labels_direction1[vertex as usize] = forward_label;
}

pub fn prune_label(
    label_direction1: &mut Vec<HubLabelEntry>,
    labels_direction2: &Vec<Vec<HubLabelEntry>>,
) {
    let mut new_label = label_direction1
        .iter()
        .filter(|entry| {
            let other_label = labels_direction2.get(entry.vertex as usize).unwrap();
            let true_distance = overlapp(label_direction1, other_label).unwrap().0;

            entry.distance == true_distance
        })
        .cloned()
        .collect::<Vec<_>>();

    std::mem::swap(&mut new_label, label_direction1);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HubLabelEntry {
    pub vertex: Vertex,
    pub distance: Distance,
    /// index of predecessor. None if no predecessor.
    pub predecessor_index: Option<u32>,
}

impl HubLabelEntry {
    pub fn new(vertex: Vertex) -> Self {
        HubLabelEntry {
            vertex,
            distance: 0,
            predecessor_index: None,
        }
    }
}

pub fn overlapp(
    forward_label: &[HubLabelEntry],
    backward_label: &[HubLabelEntry],
) -> Option<(Distance, (usize, usize))> {
    let mut overlapp = None;

    let mut forward_index = 0;
    let mut backward_index = 0;

    while forward_index < forward_label.len() && backward_index < backward_label.len() {
        let forward_vertex = forward_label[forward_index].vertex;
        let backward_vertex = backward_label[backward_index].vertex;

        match forward_vertex.cmp(&backward_vertex) {
            Ordering::Less => {
                forward_index += 1;
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
            }
            Ordering::Greater => {
                backward_index += 1;
            }
        }
    }

    overlapp
}
