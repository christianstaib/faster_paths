use std::collections::HashSet;

use super::collections::{
    dijkstra_data::DijkstraData, vertex_distance_queue::VertexDistanceQueue,
    vertex_expanded_data::VertexExpandedData,
};
use crate::graphs::{Distance, Graph, Vertex};

pub fn dijktra_one_to_all(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    source: Vertex,
) {
    dijktra_one_to_one(graph, data, expanded, queue, source, Vertex::MAX)
}

pub fn dijktra_one_to_one(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    source: Vertex,
    target: Vertex,
) {
    data.set_distance(source, 0);
    queue.insert(source, 0);

    while let Some(tail) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }
        if tail == target {
            break;
        }

        let distance_tail = data.get_distance(tail).unwrap();

        for edge in graph.edges(tail) {
            let current_distance_head = data.get_distance(edge.head).unwrap_or(Distance::MAX);
            let alternative_distance_head = distance_tail + edge.weight;
            if alternative_distance_head < current_distance_head {
                data.set_distance(edge.head, alternative_distance_head);
                data.set_predecessor(edge.head, tail);
                queue.insert(edge.head, alternative_distance_head);
            }
        }
    }
}

pub fn dijktra_one_to_many(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    source: Vertex,
    targets: &Vec<Vertex>,
) {
    let mut targets: HashSet<Vertex> = targets.iter().cloned().collect();

    data.set_distance(source, 0);
    queue.insert(source, 0);

    while let Some(tail) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }
        targets.remove(&tail);
        if targets.is_empty() {
            break;
        }

        let distance_tail = data.get_distance(tail).unwrap();

        for edge in graph.edges(tail) {
            let current_distance_head = data.get_distance(edge.head).unwrap_or(Distance::MAX);
            let alternative_distance_head = distance_tail + edge.weight;
            if alternative_distance_head < current_distance_head {
                data.set_distance(edge.head, alternative_distance_head);
                data.set_predecessor(edge.head, tail);
                queue.insert(edge.head, alternative_distance_head);
            }
        }
    }
}
