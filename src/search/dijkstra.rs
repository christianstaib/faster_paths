use std::collections::HashSet;

use indicatif::ParallelProgressIterator;
use rand::{thread_rng, Rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::{
    collections::{
        dijkstra_data::{DijkstraData, DijkstraDataHashMap, DijkstraDataVec, Path},
        vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
        vertex_expanded_data::{
            VertexExpandedData, VertexExpandedDataBitSet, VertexExpandedDataHashSet,
        },
    },
    path::{ShortestPathRequest, ShortestPathTestCase},
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

    while let Some((tail, distance_tail)) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }

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
pub fn dijkstra_one_to_all_wraped(graph: &dyn Graph, source: Vertex) -> DijkstraDataVec {
    let mut data = DijkstraDataVec::new(graph);
    let mut expanded = VertexExpandedDataBitSet::new(graph);
    let mut queue = VertexDistanceQueueBinaryHeap::new();
    dijktra_one_to_all(graph, &mut data, &mut expanded, &mut queue, source);
    data
}

pub fn dijkstra_one_to_one(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    source: Vertex,
    target: Vertex,
) {
    data.set_distance(source, 0);
    queue.insert(source, 0);

    while let Some((tail, distance_tail)) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }
        if tail == target {
            break;
        }

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
pub fn dijkstra_one_to_one_wrapped(
    graph: &dyn Graph,
    source: Vertex,
    target: Vertex,
) -> Option<Path> {
    let mut data = DijkstraDataVec::new(graph);
    let mut expanded = VertexExpandedDataBitSet::new(graph);
    let mut queue = VertexDistanceQueueBinaryHeap::new();
    dijkstra_one_to_one(graph, &mut data, &mut expanded, &mut queue, source, target);
    data.get_path(target)
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

    while let Some((tail, distance_tail)) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }
        targets.remove(&tail);
        if targets.is_empty() {
            break;
        }

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

pub fn create_test_cases(graph: &dyn Graph, number_of_testcases: u32) -> Vec<ShortestPathTestCase> {
    (0..number_of_testcases)
        .into_par_iter()
        .progress()
        .map_init(
            || {
                (
                    DijkstraDataVec::new(graph),
                    VertexExpandedDataBitSet::new(graph),
                    VertexDistanceQueueBinaryHeap::new(),
                    thread_rng(),
                )
            },
            |(data, expanded, queue, rng), _| {
                let source = rng.gen_range(0..graph.number_of_vertices());
                let target = rng.gen_range(0..graph.number_of_vertices());
                dijkstra_one_to_one(graph, data, expanded, queue, source, target);
                let distance = data.get_distance(target);

                data.clear();
                expanded.clear();
                queue.clear();

                ShortestPathTestCase {
                    source,
                    target,
                    distance,
                }
            },
        )
        .collect()
}
