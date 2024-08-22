use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    graphs::{vec_vec_graph::VecVecGraph, Distance, Graph, Vertex, WeightedEdge},
    search::{
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataHashMap},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataHashSet},
        },
        dijkstra::dijkstra_one_to_all_wraped,
    },
};

#[derive(Serialize, Deserialize)]
pub struct ContractedGraph {
    pub upward_graph: VecVecGraph,
    pub downward_graph: VecVecGraph,
    pub level_to_vertex: Vec<Vertex>,
    pub vertex_to_level: Vec<u32>,
}

impl ContractedGraph {
    pub fn new(
        edges: HashMap<(Vertex, Vertex), Distance>,
        level_to_vertex: &Vec<u32>,
    ) -> ContractedGraph {
        let vertex_to_level = vertex_to_level(&level_to_vertex);

        let mut upward_edges = Vec::new();
        let mut downward_edges = Vec::new();
        for (&(tail, head), &weight) in edges.iter() {
            if vertex_to_level[tail as usize] < vertex_to_level[head as usize] {
                upward_edges.push(WeightedEdge::new(tail, head, weight));
            } else if vertex_to_level[tail as usize] > vertex_to_level[head as usize] {
                downward_edges.push(WeightedEdge::new(head, tail, weight));
            }
        }

        ContractedGraph {
            upward_graph: VecVecGraph::from_edges(&upward_edges),
            downward_graph: VecVecGraph::from_edges(&downward_edges),
            level_to_vertex: level_to_vertex.clone(),
            vertex_to_level,
        }
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
) -> Option<Distance> {
    let mut forward_data = DijkstraDataHashMap::new();
    let mut forward_expanded = VertexExpandedDataHashSet::new();
    let mut forward_queue = VertexDistanceQueueBinaryHeap::new();

    let mut backward_data = DijkstraDataHashMap::new();
    let mut backward_expanded = VertexExpandedDataHashSet::new();
    let mut backward_queue = VertexDistanceQueueBinaryHeap::new();

    ch_one_to_one(
        ch_graph,
        &mut forward_data,
        &mut forward_expanded,
        &mut forward_queue,
        &mut backward_data,
        &mut backward_expanded,
        &mut backward_queue,
        source,
        target,
    )
    .map(|(_vertex, distance)| distance)
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
