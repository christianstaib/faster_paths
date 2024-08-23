use std::collections::HashMap;

use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::hub_graph::{HubGraph, HubLabelEntry};
use crate::{
    graphs::{reversible_graph::ReversibleGraph, Distance, Graph, Vertex},
    search::collections::{
        dijkstra_data::{DijkstraData, DijkstraDataVec},
        vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
        vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
    },
};

pub fn brute_force<G: Graph + Default>(
    graph: &ReversibleGraph<G>,
    vertex_to_level: &Vec<u32>,
) -> HubGraph {
    let (forward_labels, forward_indices) = half_brute_force(graph.out_graph(), vertex_to_level);
    let (backward_labels, backward_indices) = half_brute_force(graph.in_graph(), vertex_to_level);

    HubGraph {
        forward_labels,
        forward_indices,
        backward_labels,
        backward_indices,
    }
}

fn half_brute_force(
    graph: &dyn Graph,
    vertex_to_level: &Vec<u32>,
) -> (Vec<HubLabelEntry>, Vec<(u32, u32)>) {
    let forward_labels = (0..graph.number_of_vertices())
        .into_par_iter()
        .progress()
        .map_init(
            || {
                (
                    DijkstraDataVec::new(graph),
                    VertexExpandedDataBitSet::new(graph),
                    VertexDistanceQueueBinaryHeap::new(),
                )
            },
            |(data, expanded, queue), vertex| {
                let label = get_hub_label(graph, data, expanded, queue, vertex_to_level, vertex);

                data.clear();
                expanded.clear();
                queue.clear();

                label
            },
        )
        .collect::<Vec<_>>();

    let forward_indices: Vec<(u32, u32)> = forward_labels
        .iter()
        .map(|label| label.len() as u32)
        .scan(0, |state, len| {
            let start = *state;
            *state += len;
            Some((start, *state))
        })
        .collect();

    (
        forward_labels.into_iter().flatten().collect_vec(),
        forward_indices,
    )
}

pub fn get_hub_label(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    vertex_to_level: &Vec<u32>,
    source: Vertex,
) -> Vec<HubLabelEntry> {
    // Maps (vertex -> (max level on path from source to vertex, associated vertex))
    //
    // A vertex is a head of a ch edge if its levels equals the max level on its
    // path from the source. The tail of this ch edge is is the vertex with the
    // max level on the path to the head's predecessor
    let mut max_level_on_path = HashMap::new();
    max_level_on_path.insert(source, (vertex_to_level[source as usize], source));

    data.set_distance(source, 0);
    queue.insert(source, 0);

    let mut hub_label = vec![HubLabelEntry {
        vertex: source,
        distance: 0,
        predecessor_index: None,
    }];

    while let Some((tail, distance_tail)) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }

        let (max_level_tail, max_level_tail_vertex) = max_level_on_path[&tail];
        let level_tail = vertex_to_level[tail as usize];

        // Check if tail is a head of a ch edge
        // And dont create a edge from source to source
        if (max_level_tail == level_tail) && (tail != source) {
            let predecessor = data.get_predecessor(tail).unwrap();
            let edge_tail = max_level_on_path.get(&predecessor).unwrap().1;

            // Only add edge if its tail is source. This function only returns edges with a
            // tail in source.
            hub_label.push(HubLabelEntry {
                vertex: tail,
                distance: data.get_distance(tail).unwrap(),
                predecessor_index: Some(edge_tail),
            });
        }

        for edge in graph.edges(tail) {
            let current_distance_head = data.get_distance(edge.head).unwrap_or(Distance::MAX);
            let alternative_distance_head = distance_tail + edge.weight;
            if alternative_distance_head < current_distance_head {
                data.set_distance(edge.head, alternative_distance_head);
                data.set_predecessor(edge.head, tail);
                queue.insert(edge.head, alternative_distance_head);

                let level_head = vertex_to_level[edge.head as usize];
                if level_head > max_level_tail {
                    max_level_on_path.insert(edge.head, (level_head, edge.head));
                } else {
                    max_level_on_path.insert(edge.head, (max_level_tail, max_level_tail_vertex));
                }
            }
        }
    }

    hub_label.sort_by_key(|entry| entry.vertex);

    let vertex_to_index = hub_label
        .iter()
        .enumerate()
        .map(|(index, entry)| (entry.vertex, index as u32))
        .collect::<HashMap<_, _>>();

    hub_label.iter_mut().for_each(|entry| {
        if let Some(ref mut predecessor) = entry.predecessor_index {
            *predecessor = *vertex_to_index.get(&predecessor).unwrap();
        }
    });

    hub_label
}
