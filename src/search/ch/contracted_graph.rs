use std::collections::HashMap;

use indicatif::ProgressIterator;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::{
    brute_force::brute_force_contracted_graph_edges, contraction::contraction_with_witness_search,
};
use crate::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Distance, Graph, Vertex,
        WeightedEdge,
    },
    search::{
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataHashMap, Path},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataHashSet},
        },
        dijkstra::dijkstra_one_to_all_wraped,
    },
};

#[derive(Serialize, Deserialize)]
pub struct ContractedGraph {
    upward_graph: VecVecGraph,
    downward_graph: VecVecGraph,
    shortcuts: HashMap<(Vertex, Vertex), Vertex>,
    level_to_vertex: Vec<Vertex>,
    vertex_to_level: Vec<u32>,
}

pub fn get_slow_shortcuts(
    edges_and_predecessors: &Vec<(WeightedEdge, Option<Vertex>)>,
) -> HashMap<(Vertex, Vertex), Vertex> {
    let mut shortcuts: HashMap<(Vertex, Vertex), Vertex> = HashMap::new();

    for (edge, predecessor) in edges_and_predecessors.iter() {
        if let Some(predecessor) = predecessor {
            shortcuts.insert((edge.tail, edge.head), *predecessor);
        }
    }

    shortcuts
}

pub fn get_fast_shortcuts(
    single_step_shortcuts: &HashMap<(Vertex, Vertex), Vertex>,
) -> HashMap<(Vertex, Vertex), Vec<Vertex>> {
    let mut shortcuts = HashMap::new();

    for (&(tail, head), &vertex) in single_step_shortcuts.iter() {
        let mut path = vec![tail, vertex, head];
        path.remove(0); // remove tail
        path.pop(); // remove head
        replace_shortcuts_slowly(&mut path, &single_step_shortcuts);
        shortcuts.insert((tail, head), path);
    }

    shortcuts
}

pub fn replace_shortcuts_slowly(
    path_with_shortcuts: &mut Vec<Vertex>,
    shortcuts: &HashMap<(Vertex, Vertex), Vertex>,
) {
    let mut path_without_shortcuts = Vec::new();

    while path_with_shortcuts.len() >= 2 {
        let head = path_with_shortcuts.pop().unwrap();
        let tail = *path_with_shortcuts.last().unwrap();
        if let Some(vertex) = shortcuts.get(&(tail, head)) {
            path_with_shortcuts.push(*vertex);
            path_with_shortcuts.push(head);
        } else {
            path_without_shortcuts.push(head);
        }
    }
    path_without_shortcuts.push(path_with_shortcuts.pop().unwrap());
    path_without_shortcuts.reverse();

    *path_with_shortcuts = path_without_shortcuts;
}

pub fn replace_shortcuts_fast(
    path_with_shortcuts: &mut Vec<Vertex>,
    shortcuts: &HashMap<(Vertex, Vertex), Vec<Vertex>>,
) {
    let mut path_without_shortcuts = vec![*path_with_shortcuts.first().unwrap()];

    for (&tail, &head) in path_with_shortcuts.iter().tuple_windows() {
        if let Some(skiped_vertices) = shortcuts.get(&(tail, head)) {
            path_without_shortcuts.extend(skiped_vertices);
        }
        path_without_shortcuts.push(head)
    }

    *path_with_shortcuts = path_without_shortcuts;
}

impl ContractedGraph {
    pub fn by_contraction_with_dijkstra_witness_search<G: Graph + Default + Clone>(
        graph: &ReversibleGraph<G>,
    ) -> ContractedGraph {
        let graph = graph.clone();
        let (level_to_vertex, edges, shortcuts) = contraction_with_witness_search(graph);

        let vertex_to_level = vertex_to_level(&level_to_vertex);

        let mut upward_edges = Vec::new();
        let mut downward_edges = Vec::new();
        for (&(tail, head), &weight) in edges.iter().progress() {
            if vertex_to_level[tail as usize] < vertex_to_level[head as usize] {
                upward_edges.push(WeightedEdge::new(tail, head, weight));
            } else if vertex_to_level[tail as usize] > vertex_to_level[head as usize] {
                downward_edges.push(WeightedEdge::new(head, tail, weight));
            }
        }

        ContractedGraph {
            upward_graph: VecVecGraph::from_edges(&upward_edges),
            downward_graph: VecVecGraph::from_edges(&downward_edges),
            shortcuts,
            level_to_vertex: level_to_vertex.clone(),
            vertex_to_level,
        }
    }

    pub fn by_brute_force<G: Graph + Default>(
        graph: &ReversibleGraph<G>,
        level_to_vertex: &Vec<u32>,
    ) -> ContractedGraph {
        let vertex_to_level = vertex_to_level(&level_to_vertex);

        let (upward_edges, upward_shortcuts) =
            brute_force_contracted_graph_edges(graph.out_graph(), &vertex_to_level);

        let (downward_edges, downward_shortcuts) =
            brute_force_contracted_graph_edges(graph.in_graph(), &vertex_to_level);

        let mut shortcuts = HashMap::new();
        shortcuts.extend(upward_shortcuts);
        shortcuts.extend(downward_shortcuts);

        ContractedGraph {
            upward_graph: VecVecGraph::from_edges(&upward_edges),
            downward_graph: VecVecGraph::from_edges(&downward_edges),
            shortcuts,
            level_to_vertex: level_to_vertex.clone(),
            vertex_to_level,
        }
    }

    pub fn upward_graph(&self) -> &dyn Graph {
        &self.upward_graph
    }

    pub fn downward_graph(&self) -> &dyn Graph {
        &self.downward_graph
    }

    pub fn level_to_vertex(&self) -> &Vec<Vertex> {
        &self.level_to_vertex
    }

    pub fn vertex_to_level(&self) -> &Vec<u32> {
        &self.vertex_to_level
    }

    pub fn shortest_path_distance(&self, source: Vertex, target: Vertex) -> Option<Distance> {
        let up_weights = dijkstra_one_to_all_wraped(&self.upward_graph, source);
        let down_weights = dijkstra_one_to_all_wraped(&self.downward_graph, target);

        let mut min_distance = Distance::MAX;
        for vertex in 0..std::cmp::max(
            self.upward_graph.number_of_vertices(),
            self.downward_graph.number_of_vertices(),
        ) {
            let alt_distance = match (
                up_weights.get_distance(vertex),
                down_weights.get_distance(vertex),
            ) {
                (Some(a), Some(b)) => a + b,
                _ => Distance::MAX,
            };

            if alt_distance < min_distance {
                min_distance = alt_distance;
            }
        }

        if min_distance == Distance::MAX {
            return None;
        }

        Some(min_distance)
    }
}

pub fn vertex_to_level(level_to_vertex: &Vec<Vertex>) -> Vec<u32> {
    let mut vertex_to_level = vec![0; level_to_vertex.len()];

    for (level, &vertex) in level_to_vertex.iter().enumerate() {
        vertex_to_level[vertex as usize] = level as u32;
    }

    vertex_to_level
}

pub fn ch_one_to_one_wrapped(
    ch_graph: &ContractedGraph,
    source: Vertex,
    target: Vertex,
) -> Option<Path> {
    let mut forward_data = DijkstraDataHashMap::new();
    let mut forward_expanded = VertexExpandedDataHashSet::new();
    let mut forward_queue = VertexDistanceQueueBinaryHeap::new();

    let mut backward_data = DijkstraDataHashMap::new();
    let mut backward_expanded = VertexExpandedDataHashSet::new();
    let mut backward_queue = VertexDistanceQueueBinaryHeap::new();

    let (vertex, distance) = ch_one_to_one(
        ch_graph,
        &mut forward_data,
        &mut forward_expanded,
        &mut forward_queue,
        &mut backward_data,
        &mut backward_expanded,
        &mut backward_queue,
        source,
        target,
    )?;

    let mut forward_vertices = forward_data.get_path(vertex).unwrap().vertices;
    let mut backward_vertices = backward_data.get_path(vertex).unwrap().vertices;

    replace_shortcuts_slowly(&mut forward_vertices, &ch_graph.shortcuts);
    replace_shortcuts_slowly(&mut backward_vertices, &ch_graph.shortcuts);

    forward_vertices.pop(); // remove double
    backward_vertices.reverse();

    forward_vertices.extend(backward_vertices);

    Some(Path {
        vertices: forward_vertices,
        distance,
    })
}

pub fn ch_one_to_one(
    ch_graph: &ContractedGraph,
    forward_data: &mut dyn DijkstraData,
    forward_expanded: &mut dyn VertexExpandedData,
    forward_queue: &mut dyn VertexDistanceQueue,
    backward_data: &mut dyn DijkstraData,
    backward_expanded: &mut dyn VertexExpandedData,
    backward_queue: &mut dyn VertexDistanceQueue,
    source: Vertex,
    target: Vertex,
) -> Option<(Vertex, Distance)> {
    forward_data.set_distance(source, 0);
    forward_queue.insert(source, 0);

    backward_data.set_distance(target, 0);
    backward_queue.insert(target, 0);

    let mut meeting_vertex_and_distance = None;

    while !forward_queue.is_empty() || !backward_queue.is_empty() {
        if let Some((tail, distance_tail)) = forward_queue.pop() {
            if forward_expanded.expand(tail) {
                continue;
            }

            if let Some(backward_distance_tail) = backward_data.get_distance(tail) {
                let meeting_distance = meeting_vertex_and_distance
                    .map_or(Distance::MAX, |(_vertex, distance)| distance);
                let alternative_meeting_distance = distance_tail + backward_distance_tail;
                if alternative_meeting_distance < meeting_distance {
                    meeting_vertex_and_distance = Some((tail, alternative_meeting_distance));
                }
            }

            for edge in ch_graph.upward_graph.edges(tail) {
                let current_distance_head = forward_data
                    .get_distance(edge.head)
                    .unwrap_or(Distance::MAX);
                let alternative_distance_head = distance_tail + edge.weight;
                if alternative_distance_head < current_distance_head {
                    forward_data.set_distance(edge.head, alternative_distance_head);
                    forward_data.set_predecessor(edge.head, tail);
                    forward_queue.insert(edge.head, alternative_distance_head);
                }
            }
        }

        if let Some((tail, distance_tail)) = backward_queue.pop() {
            if backward_expanded.expand(tail) {
                continue;
            }

            if let Some(forward_distance_tail) = forward_data.get_distance(tail) {
                let meeting_distance = meeting_vertex_and_distance
                    .map_or(Distance::MAX, |(_vertex, distance)| distance);
                let alternative_meeting_distance = distance_tail + forward_distance_tail;
                if alternative_meeting_distance < meeting_distance {
                    meeting_vertex_and_distance = Some((tail, alternative_meeting_distance));
                }
            }

            for edge in ch_graph.downward_graph.edges(tail) {
                let current_distance_head = backward_data
                    .get_distance(edge.head)
                    .unwrap_or(Distance::MAX);
                let alternative_distance_head = distance_tail + edge.weight;
                if alternative_distance_head < current_distance_head {
                    backward_data.set_distance(edge.head, alternative_distance_head);
                    backward_data.set_predecessor(edge.head, tail);
                    backward_queue.insert(edge.head, alternative_distance_head);
                }
            }
        }
    }

    meeting_vertex_and_distance
}
