use std::time::{Duration, Instant};

use radix_heap::RadixHeapMap;

use super::{
    dijkstra_data::DijkstraData, vertex_distance_queue::VertexDistanceQueue,
    vertex_expanded_data::VertexExpandedData,
};
use crate::graphs::{Graph, VertexId, Weight};

pub fn dijktra_single_source(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    source: VertexId,
) {
    data.set_distance(source, 0);
    queue.insert(source, 0);

    while let Some(tail) = queue.pop() {
        if expanded.expand(tail) {
            continue;
        }

        let distance_tail = data.get_distance(tail).unwrap();

        for edge in graph.edges(tail) {
            let current_distance_head = data.get_distance(edge.head).unwrap_or(Weight::MAX);
            let alternative_distance_head = distance_tail + edge.weight;
            if alternative_distance_head < current_distance_head {
                data.set_distance(edge.head, alternative_distance_head);
                data.set_predecessor(edge.head, tail);
                queue.insert(edge.head, alternative_distance_head);
            }
        }
    }
}

pub fn dijktra_single_pair(
    graph: &dyn Graph,
    data: &mut dyn DijkstraData,
    expanded: &mut dyn VertexExpandedData,
    queue: &mut dyn VertexDistanceQueue,
    source: VertexId,
    target: VertexId,
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
            let current_distance_head = data.get_distance(edge.head).unwrap_or(Weight::MAX);
            let alternative_distance_head = distance_tail + edge.weight;
            if alternative_distance_head < current_distance_head {
                data.set_distance(edge.head, alternative_distance_head);
                data.set_predecessor(edge.head, tail);
                queue.insert(edge.head, alternative_distance_head);
            }
        }
    }
}
