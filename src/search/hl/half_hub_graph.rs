use std::collections::HashMap;

use clap::Parser;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressIterator};
use itertools::Itertools;
use rayon::iter::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use super::hub_graph::{overlapp, HubLabelEntry};
use crate::{
    graphs::{Distance, Graph, Vertex, WeightedEdge},
    search::{
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataVec},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
        },
        shortcuts,
    },
};

pub struct HalfHubGraph {
    labels: Vec<HubLabelEntry>,
    indices: Vec<(u32, u32)>,
}

impl HalfHubGraph {
    pub fn new(labels: &Vec<Vec<HubLabelEntry>>) -> Self {
        let indices: Vec<(u32, u32)> = labels
            .iter()
            .map(|label| label.len() as u32)
            .scan(0, |state, len| {
                let start = *state;
                *state += len;
                Some((start, *state))
            })
            .collect();

        let labels = labels.iter().flatten().cloned().collect();

        HalfHubGraph { labels, indices }
    }

    pub fn by_brute_force(
        graph: &dyn Graph,
        vertex_to_level: &Vec<u32>,
        progress_bar: ProgressBar,
    ) -> (HalfHubGraph, HashMap<(Vertex, Vertex), Vertex>) {
        let labels_and_shortcuts = (0..graph.number_of_vertices())
            .into_par_iter()
            .progress_with(progress_bar)
            .map_init(
                || {
                    (
                        DijkstraDataVec::new(graph),
                        VertexExpandedDataBitSet::new(graph),
                        VertexDistanceQueueBinaryHeap::new(),
                    )
                },
                |(data, expanded, queue), vertex| {
                    let labels_and_shortcuts = get_hub_label_with_brute_force(
                        graph,
                        data,
                        expanded,
                        queue,
                        vertex_to_level,
                        vertex,
                    );

                    data.clear();
                    expanded.clear();
                    queue.clear();

                    labels_and_shortcuts
                },
            )
            .collect::<Vec<_>>();

        let mut all_labels = Vec::new();
        let mut all_shortcuts = HashMap::new();

        for (label, shortcuts) in labels_and_shortcuts {
            all_labels.push(label);
            all_shortcuts.extend(shortcuts);
        }

        (HalfHubGraph::new(&all_labels), all_shortcuts)
    }

    pub fn get_label(&self, vertex: Vertex) -> &[HubLabelEntry] {
        let &(start, stop) = self.indices.get(vertex as usize).unwrap_or(&(0, 0));
        &self.labels[start as usize..stop as usize]
    }

    pub fn average_label_size(&self) -> f32 {
        self.labels.len() as f32 / self.indices.len() as f32
    }
}

pub fn get_hub_label_with_brute_force(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    vertex_to_level: &Vec<u32>,
    source: Vertex,
) -> (Vec<HubLabelEntry>, Vec<((Vertex, Vertex), Vertex)>) {
    // Maps (vertex -> (max level on path from source to vertex, associated vertex))
    //
    // A vertex is a head of a ch edge if its levels equals the max level on its
    // path from the source. The tail of this ch edge is is the vertex with the
    // max level on the path to the head's predecessor
    let mut max_level_on_path = HashMap::new();
    max_level_on_path.insert(source, (vertex_to_level[source as usize], source));

    data.set_distance(source, 0);
    queue.insert(source, 0);

    let mut shortcuts = Vec::new();

    let mut hub_label = vec![HubLabelEntry::new(source)];

    while let Some((tail, distance_tail)) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }

        let (max_level_tail, max_level_tail_vertex) = max_level_on_path[&tail];
        let level_tail = vertex_to_level[tail as usize];

        // Check if tail is a head of a ch edge
        if max_level_tail == level_tail {
            // for less confusion, rename variables

            // Dont create a edge from source to source. source has no predecessor
            if let Some(predecessor) = data.get_predecessor(tail) {
                let entry_predecessor = max_level_on_path.get(&predecessor).unwrap().1;
                let shortcut_head = tail;

                // Only add edge if its tail is source. This function only returns edges with a
                // tail in source.
                hub_label.push(HubLabelEntry {
                    vertex: tail,
                    distance: data.get_distance(tail).unwrap(),
                    predecessor_index: Some(entry_predecessor),
                });

                if entry_predecessor == source {
                    let mut path = data.get_path(shortcut_head).unwrap();
                    path.vertices.remove(0);
                    path.vertices.pop();

                    path.vertices
                        .iter()
                        .max_by_key(|&&vertex| vertex_to_level[vertex as usize])
                        .map(|&skiped_vertex| {
                            shortcuts.push(((entry_predecessor, shortcut_head), skiped_vertex))
                        });
                }
            }
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

    set_predecessor(&mut hub_label);

    (hub_label, shortcuts)
}

pub fn set_predecessor(hub_label: &mut Vec<HubLabelEntry>) {
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
}

pub fn get_hub_label_by_merging(
    labels: &Vec<(Option<WeightedEdge>, &Vec<HubLabelEntry>)>,
) -> Vec<HubLabelEntry> {
    let mut this_edge = None;
    let mut new_label = Vec::new();

    let mut labels = labels
        .iter()
        .map(|(edge, label)| (edge, label.iter().peekable()))
        .collect_vec();

    while !labels.is_empty() {
        let mut min_entry = HubLabelEntry {
            vertex: Vertex::MAX,
            distance: Distance::MAX,
            predecessor_index: None,
        };

        let mut labels_with_min_vertex = Vec::new();

        for (edge, label) in labels.iter_mut() {
            let edge = edge.as_ref();
            if this_edge.is_none() {
                this_edge = edge.clone();
            } else {
            }
            let entry = *label.peek().unwrap();

            if entry.vertex <= min_entry.vertex {
                if entry.vertex < min_entry.vertex {
                    min_entry.vertex = entry.vertex;
                    min_entry.distance = Distance::MAX;
                    min_entry.predecessor_index = None;

                    labels_with_min_vertex.clear();
                }

                let alternative_distance =
                    entry.distance + edge.as_ref().map(|edge| edge.weight).unwrap_or(0);
                if alternative_distance < min_entry.distance {
                    min_entry.distance = alternative_distance;
                    if entry.predecessor_index.is_none() && edge.is_some() {
                        min_entry.predecessor_index = Some(edge.as_ref().unwrap().tail);
                    } else {
                        min_entry.predecessor_index = entry.predecessor_index;
                    }
                }
                labels_with_min_vertex.push(label);
            }
        }

        new_label.push(min_entry);

        labels_with_min_vertex.iter_mut().for_each(|label| {
            label.next();
        });

        // Retain only non-empty iterators
        labels.retain_mut(|(_edge, label)| label.peek().is_some());
    }

    new_label
}
