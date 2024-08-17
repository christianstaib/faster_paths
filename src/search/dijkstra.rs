use std::collections::HashSet;

use super::collections::{
    dijkstra_data::{DijkstraData, DijkstraDataHashMap, DijkstraDataVec},
    vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
    vertex_expanded_data::{
        VertexExpandedData, VertexExpandedDataBitSet, VertexExpandedDataHashSet,
    },
};
use crate::graphs::{Distance, Graph, Vertex};

/// requires data, expanded and queue to be cleared before calling.
pub fn dijktra_one_to_all(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    source: Vertex,
) {
    data.set_distance(source, 0);
    queue.insert(source, 0);

    while let Some(tail) = queue.pop() {
        if expanded.expand(tail) {
            continue;
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

/// Wrapper that creates all nesseary data structures each time when called
/// which can have a performance malus.
pub fn dijkstra_one_to_all(graph: &dyn Graph, source: Vertex) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph);
    let mut expanded = VertexExpandedDataBitSet::new(graph);
    let mut queue = VertexDistanceQueueBinaryHeap::new();
    dijktra_one_to_all(graph, &mut data, &mut expanded, &mut queue, source);
    data
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

pub fn dijkstra_one_to_many(
    graph: &dyn Graph,
    source: Vertex,
    targets: &Vec<Vertex>,
) -> DijkstraDataHashMap {
    let mut data = DijkstraDataHashMap::new();
    let mut expanded = VertexExpandedDataHashSet::new();
    let mut queue = VertexDistanceQueueBinaryHeap::new();
    dijktra_one_to_many(graph, &mut data, &mut expanded, &mut queue, source, targets);
    data
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
