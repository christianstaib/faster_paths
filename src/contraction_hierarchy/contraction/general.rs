use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::contraction_hierarchy::ContractionEdge;
use crate::graph::{DirectionalAdjacencyListGraph, EdgeLike, GraphLike};
use crate::types::Distance;
use crate::types::Vertex;

/// Calculates the edge difference. Used in order to avoid calculation errors if always written out.
pub(super) fn edge_difference<D: Distance>(
    graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    vertex: Vertex,
    shortcut_count: usize,
) -> i64 {
    let degree =
        graph.forward().outgoing_edges(vertex).len() + graph.reverse().outgoing_edges(vertex).len();
    shortcut_count as i64 - degree as i64
}

/// Given normal edge-like values, build a `WorkingGraph` which is used during contraction.
pub fn build_working_graph<'a, E>(
    edges: impl IntoIterator<Item = &'a E>,
) -> DirectionalAdjacencyListGraph<ContractionEdge<E::Weight>>
where
    E: EdgeLike + 'a,
{
    let mut working_graph = DirectionalAdjacencyListGraph::new();

    edges.into_iter().for_each(|edge| {
        let contraction_edge = ContractionEdge {
            tail: edge.tail(),
            head: edge.head(),
            weight: edge.weight(),
            skipped: None,
        };

        working_graph.add_edge(&contraction_edge);
    });

    working_graph
}

/// Computes the shortcuts necessary to maintain the shortest path distances in `graph` if vertex
/// would be disconnected and also possibly some more.
///
/// A shortcut u -> w for u -> v -> w is necessary iff (u, v, w) is the only shortest u-v-path.
/// This function relaxes this condition by limiting the search space size with max_hops.
pub(super) fn generate_shortcuts<D: Distance>(
    graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    vertex: Vertex,
    max_hops: u32,
) -> Vec<ContractionEdge<D>> {
    let outgoing_edges = graph.forward().outgoing_edges(vertex);
    let targets = outgoing_edges
        .iter()
        .map(|edge| edge.head)
        .collect::<FxHashSet<_>>();

    graph
        .reverse()
        .outgoing_edges(vertex)
        .iter()
        .flat_map(|incoming_edge| {
            let tail = incoming_edge.head;
            let tail_weight = incoming_edge.weight;
            let distances = bounded_dijkstra(graph, tail, &targets, max_hops);

            outgoing_edges.iter().filter_map(move |edge| {
                if tail == edge.head {
                    return None;
                }

                let weight = tail_weight + edge.weight;

                distances
                    .get(&edge.head)
                    .is_none_or(|&witness_distance| witness_distance >= weight)
                    .then(|| ContractionEdge {
                        tail,
                        head: edge.head,
                        weight,
                        skipped: Some(vertex),
                    })
            })
        })
        .collect()
}

/// Computes shortest path distances from `source` to `targets`.
///
/// Stops once every target has been settled or once only vertices with hop distance > `max_hops` remain. The returned map may contain non-target vertices.
pub(super) fn bounded_dijkstra<D: Distance>(
    graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    source: Vertex,
    targets: &FxHashSet<Vertex>,
    max_hops: u32,
) -> FxHashMap<Vertex, D> {
    let mut distances = FxHashMap::default();
    let mut hops = FxHashMap::default();
    let mut queue = BinaryHeap::new();
    let mut expanded = FxHashSet::default();

    let mut remaining_targets = targets.len();

    distances.insert(source, D::zero());
    hops.insert(source, 0);
    queue.push((Reverse(D::zero()), source));

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

        for edge in graph.forward().outgoing_edges(vertex) {
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
