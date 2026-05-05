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
    collections::{BinaryHeap, HashMap},
};

const WITNESS_SEARCH_CAPACITY: usize = 256;
const MAX_WITNESS_HOPS: usize = 100;

pub fn contract(graph: &FlattenedNested<Edge>) -> ContractionHierarchy {
    let mut working_graph = build_working_graph(graph);
    let levels = contract_vertices(&mut working_graph);

    build_hierarchy(&working_graph, &levels)
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

fn contract_vertices(graph: &mut WorkingGraph) -> Vec<usize> {
    let vertex_count = graph.num_vertices();
    let mut levels = vec![0; vertex_count];
    let mut queue = initial_queue(graph);
    let progress = ProgressBar::new(vertex_count as u64);

    let mut next_level = 0;
    while let Some((Reverse(queued_difference), vertex)) = queue.pop() {
        let shortcuts = generate_shortcuts(graph, vertex);
        let difference = edge_difference(graph, vertex, shortcuts.len());

        if difference > queued_difference {
            queue.push((Reverse(difference), vertex));
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

    levels
}

fn initial_queue(graph: &WorkingGraph) -> BinaryHeap<(Reverse<i64>, VertexId)> {
    (0..graph.num_vertices())
        .into_par_iter()
        .progress()
        .map(|index| {
            let vertex = VertexId::new(index as u32);
            let shortcut_count = generate_shortcuts(graph, vertex).len();

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

fn generate_shortcuts(graph: &WorkingGraph, vertex: VertexId) -> Vec<ChEdge> {
    let mut shortcuts = Vec::new();
    let outgoing = graph.get_out(vertex);

    for &(tail, tail_weight) in graph.get_in(vertex) {
        let targets = outgoing
            .iter()
            .map(|edge| edge.head())
            .filter(|&head| tail != head)
            .collect::<Vec<_>>();

        if targets.is_empty() {
            continue;
        }

        let distances = witness_distances(graph, tail, &targets);

        for edge in outgoing {
            let head = edge.head();

            if tail == head {
                continue;
            }

            let weight = tail_weight + edge.weight();
            if distances
                .get(&head)
                .is_none_or(|&witness_distance| witness_distance >= weight)
            {
                let shortcut = ChEdge::new(tail, head, weight, Some(vertex));
                insert_shortcut(&mut shortcuts, shortcut);
            }
        }
    }

    shortcuts
}

fn witness_distances(
    graph: &WorkingGraph,
    source: VertexId,
    targets: &[VertexId],
) -> HashMap<VertexId, Distance> {
    let mut distances = HashMap::with_capacity(WITNESS_SEARCH_CAPACITY);
    let mut hops = HashMap::with_capacity(WITNESS_SEARCH_CAPACITY);
    let mut queue = BinaryHeap::with_capacity(WITNESS_SEARCH_CAPACITY);

    let mut remaining_targets = targets.len();

    distances.insert(source, Distance::ZERO);
    hops.insert(source, 0);
    queue.push((Reverse(Distance::ZERO), source));

    while let Some((Reverse(distance), vertex)) = queue.pop() {
        if distances
            .get(&vertex)
            .is_some_and(|&best_distance| distance > best_distance)
        {
            continue;
        }

        if targets.contains(&vertex) {
            remaining_targets -= 1;
            if remaining_targets == 0 {
                break;
            }
        }

        let hop_count = hops[&vertex];
        if hop_count > MAX_WITNESS_HOPS {
            continue;
        }

        for edge in graph.get_out(vertex) {
            let head = edge.head();
            let next_distance = distance + edge.weight();

            if distances
                .get(&head)
                .is_some_and(|&best_distance| best_distance <= next_distance)
            {
                continue;
            }

            distances.insert(head, next_distance);
            hops.insert(head, hop_count + 1);
            queue.push((Reverse(next_distance), head));
        }
    }

    distances
}

fn build_hierarchy(graph: &WorkingGraph, levels: &[usize]) -> ContractionHierarchy {
    let mut up = vec![Vec::new(); levels.len()];
    let mut down = vec![Vec::new(); levels.len()];

    for &edge in graph.contracted_edges() {
        let tail = edge.tail();
        let head = edge.head();
        let tail_index = tail.as_usize();
        let head_index = head.as_usize();

        if levels[tail_index] < levels[head_index] {
            up[tail_index].push(edge);
        } else {
            let edge = ChEdge::new(head, tail, edge.weight(), edge.skipped());
            down[head_index].push(edge);
        }
    }

    sort_edges_by_head(&mut up);
    sort_edges_by_head(&mut down);

    ContractionHierarchy::new(FlattenedNested::new(up), FlattenedNested::new(down))
}

fn insert_shortcut(shortcuts: &mut Vec<ChEdge>, edge: ChEdge) {
    match shortcuts
        .iter_mut()
        .find(|shortcut| shortcut.tail() == edge.tail() && shortcut.head() == edge.head())
    {
        Some(shortcut) if edge.weight() < shortcut.weight() => {
            *shortcut = edge;
        }
        Some(_) => {}
        None => shortcuts.push(edge),
    }
}

fn sort_edges_by_head(graph: &mut [Vec<ChEdge>]) {
    for edges in graph {
        edges.sort_by_key(|edge| edge.head());
    }
}
