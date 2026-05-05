use crate::{
    Edge as ChEdge,
    ch::{contraction_hierarchy::ContractionHierarchy, working_graph::WorkingGraph},
    edge::Edge,
    flattened_nested::FlattenedNested,
    types::{Distance, VertexId},
};
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap, HashSet},
};

const MAX_WITNESS_HOPS: u32 = 10;

pub fn contract(graph: &FlattenedNested<Edge>) -> ContractionHierarchy {
    let working_graph = build_working_graph(graph);

    contract_vertices_sequential(working_graph)
}

fn build_working_graph(graph: &FlattenedNested<Edge>) -> WorkingGraph {
    let mut working_graph = WorkingGraph::new(graph.num_nested());

    for bucket in 0..graph.num_nested() {
        for edge in graph.nested(bucket) {
            working_graph.add_edge(ChEdge::new(edge.tail(), edge.head(), edge.weight(), None));
        }
    }

    working_graph
}

fn contract_vertices_sequential(mut graph: WorkingGraph) -> ContractionHierarchy {
    let mut levels = vec![0; graph.num_vertices()];

    let mut queue = initial_queue(&graph);
    let progress = ProgressBar::new(queue.len() as u64);

    let mut next_level = 0;
    while let Some((Reverse(queued_edge_difference), vertex)) = queue.pop() {
        let shortcuts = generate_shortcuts(&graph, vertex, MAX_WITNESS_HOPS);

        let current_edge_difference = edge_difference(&graph, vertex, shortcuts.len());
        if current_edge_difference > queued_edge_difference {
            queue.push((Reverse(current_edge_difference), vertex));
            continue;
        }

        graph.contract_vertex(vertex);
        for shortcut in shortcuts {
            graph.add_edge(shortcut);
        }

        levels[vertex.as_usize()] = next_level;
        next_level += 1;
        progress.inc(1);
    }
    progress.finish();

    build_hierarchy(&graph, &levels)
}

fn build_hierarchy(graph: &WorkingGraph, levels: &[usize]) -> ContractionHierarchy {
    let mut up_graph = vec![Vec::new(); levels.len()];
    let mut down_graph = vec![Vec::new(); levels.len()];

    for &edge in graph.contracted_edges() {
        if levels[edge.tail().as_usize()] < levels[edge.head().as_usize()] {
            up_graph[edge.tail().as_usize()].push(edge);
        } else {
            down_graph[edge.head().as_usize()].push(edge.reversed());
        }
    }

    up_graph
        .iter_mut()
        .chain(down_graph.iter_mut())
        .for_each(|edges| edges.sort_by_key(|edge| edge.head()));

    ContractionHierarchy::new(
        FlattenedNested::new(up_graph),
        FlattenedNested::new(down_graph),
    )
}

fn initial_queue(graph: &WorkingGraph) -> BinaryHeap<(Reverse<i64>, VertexId)> {
    (0..graph.num_vertices() as u32)
        .into_par_iter()
        .progress()
        .map(VertexId::new)
        .map(|vertex| {
            let shortcut_count = generate_shortcuts(graph, vertex, MAX_WITNESS_HOPS).len();

            (
                Reverse(edge_difference(graph, vertex, shortcut_count)),
                vertex,
            )
        })
        .collect()
}

fn edge_difference(graph: &WorkingGraph, vertex: VertexId, shortcut_count: usize) -> i64 {
    let degree = graph.get_out(vertex).len() + graph.get_in(vertex).len();
    shortcut_count as i64 - degree as i64
}

/// Computes the shortcuts necessary to maintain the shortest path distances in `graph` if vertex
/// would be disconnected and also possibly some more.
///
/// A shortcut u -> w for u -> v -> w is necessary iff (u, v, w) is the only shortest u-v-path.
/// This function relaxes this condition by limiting the search space size with max_hops.
fn generate_shortcuts(graph: &WorkingGraph, vertex: VertexId, max_hops: u32) -> Vec<ChEdge> {
    let out_edges = graph.get_out(vertex);

    let targets = out_edges
        .iter()
        .map(|edge| edge.head())
        .collect::<HashSet<_>>();

    graph
        .get_in(vertex)
        .iter()
        .flat_map(|&(tail, tail_weight)| {
            let distances = bounded_dijkstra(graph, tail, &targets, max_hops);

            out_edges.iter().filter_map(move |edge| {
                let head = edge.head();

                if tail == head {
                    return None;
                }

                let weight = tail_weight + edge.weight();

                distances
                    .get(&head)
                    .is_none_or(|&witness_distance| witness_distance >= weight)
                    .then(|| ChEdge::new(tail, head, weight, Some(vertex)))
            })
        })
        .collect()
}

/// Computes shortest path distances from `source` to `targets`.
///
/// Stops once every target has been settled or once only vertices with hop distance > `max_hops` remain. The returned map may contain non-target vertices.
fn bounded_dijkstra(
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
            let new_distance = distance + edge.weight();

            if distances
                .get(&edge.head())
                .is_some_and(|&best_distance| new_distance >= best_distance)
            {
                continue;
            }

            distances.insert(edge.head(), new_distance);
            hops.insert(edge.head(), hop_count + 1);
            queue.push((Reverse(new_distance), edge.head()));
        }
    }

    distances
}
