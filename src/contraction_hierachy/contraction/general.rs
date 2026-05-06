use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::contraction_hierachy::ContractionEdge;
use crate::contraction_hierachy::contraction::working_graph::WorkingGraph;
use crate::contraction_hierachy::contraction_hierarchy::ContractionHierarchy;
use crate::graph::{Edge, FastGraph};
use crate::types::Distance;
use crate::{flattened_nested::FlattenedNested, types::VertexId};

/// Calculates the edge difference. Used in order to avoid calculation errors if always written out.
pub(super) fn edge_difference(
    graph: &WorkingGraph,
    vertex: VertexId,
    shortcut_count: usize,
) -> i64 {
    let degree = graph.get_out(vertex).len() + graph.get_in(vertex).len();
    shortcut_count as i64 - degree as i64
}

/// Given a normal graph, build a `WorkingGraph` which is used during contraction.
pub(super) fn build_working_graph(graph: &FlattenedNested<Edge>) -> WorkingGraph {
    let mut working_graph = WorkingGraph::new(graph);

    for bucket in 0..graph.num_nested() {
        for edge in graph.nested(bucket) {
            working_graph.add_edge(ContractionEdge::new(
                edge.tail,
                edge.head,
                edge.weight,
                None,
            ));
        }
    }

    working_graph
}

/// Given a list of `edges` which contains both up and down edges, separate them and build a
/// contraction hierarchy.
pub(super) fn build_hierarchy(edges: &[ContractionEdge], levels: &[usize]) -> ContractionHierarchy {
    let mut up_graph = vec![Vec::new(); levels.len()];
    let mut down_graph = vec![Vec::new(); levels.len()];

    for &edge in edges {
        if levels[edge.tail.as_usize()] < levels[edge.head.as_usize()] {
            up_graph[edge.tail.as_usize()].push(edge);
        } else {
            down_graph[edge.head.as_usize()].push(edge.reversed());
        }
    }

    up_graph
        .iter_mut()
        .chain(down_graph.iter_mut())
        .for_each(|edges| edges.sort_by_key(|edge| edge.head));

    ContractionHierarchy::new(FastGraph::new(&up_graph), FastGraph::new(&down_graph))
}

/// Computes the shortcuts necessary to maintain the shortest path distances in `graph` if vertex
/// would be disconnected and also possibly some more.
///
/// A shortcut u -> w for u -> v -> w is necessary iff (u, v, w) is the only shortest u-v-path.
/// This function relaxes this condition by limiting the search space size with max_hops.
pub(super) fn generate_shortcuts(
    graph: &WorkingGraph,
    vertex: VertexId,
    max_hops: u32,
) -> Vec<ContractionEdge> {
    let targets = graph
        .get_out(vertex)
        .iter()
        .map(|edge| edge.head)
        .collect::<HashSet<_>>();

    graph
        .get_in(vertex)
        .iter()
        .flat_map(|&(tail, tail_weight)| {
            let distances = bounded_dijkstra(graph, tail, &targets, max_hops);

            graph.get_out(vertex).iter().filter_map(move |edge| {
                if tail == edge.head {
                    return None;
                }

                let weight = tail_weight + edge.weight;

                distances
                    .get(&edge.head)
                    .is_none_or(|&witness_distance| witness_distance >= weight)
                    .then(|| ContractionEdge::new(tail, edge.head, weight, Some(vertex)))
            })
        })
        .collect()
}

/// Computes shortest path distances from `source` to `targets`.
///
/// Stops once every target has been settled or once only vertices with hop distance > `max_hops` remain. The returned map may contain non-target vertices.
pub(super) fn bounded_dijkstra(
    graph: &WorkingGraph,
    source: VertexId,
    targets: &HashSet<VertexId>,
    max_hops: u32,
) -> HashMap<VertexId, Distance> {
    let mut distances = HashMap::new();
    let mut hops = HashMap::new();
    let mut queue = BinaryHeap::new();
    let mut expanded = HashSet::new();

    let mut remaining_targets = targets.len();

    distances.insert(source, Distance::ZERO);
    hops.insert(source, 0);
    queue.push((Reverse(Distance::ZERO), source));

    while let Some((Reverse(distance), vertex)) = queue.pop() {
        if !expanded.insert(vertex) {
            continue;
        }

        if targets.contains(&vertex) {
            remaining_targets -= 1;
            if remaining_targets == 0 {
                break;
            }
        }

        let hop_count = hops[&vertex];
        if hop_count > max_hops {
            continue;
        }

        for edge in graph.get_out(vertex) {
            let new_distance = distance + edge.weight;

            if distances
                .get(&edge.head)
                .is_some_and(|&best_distance| new_distance >= best_distance)
            {
                continue;
            }

            distances.insert(edge.head, new_distance);
            hops.insert(edge.head, hop_count + 1);
            queue.push((Reverse(new_distance), edge.head));
        }
    }

    distances
}
