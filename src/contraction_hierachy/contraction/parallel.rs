use crate::{
    contraction_hierachy::{
        ContractionEdge,
        contraction::{
            general::{build_working_graph, edge_difference, generate_shortcuts},
            working_graph::WorkingGraph,
        },
        contraction_hierarchy::ContractionHierarchy,
    },
    graph::{EdgeLike, FastGraph, GraphLike},
    types::{Distance, VertexId},
};
use indicatif::ProgressBar;
use num_traits::clamp;
use rayon::prelude::*;
use rustc_hash::FxHashSet;
use std::time::{Duration, Instant};

const MAX_WITNESS_HOPS: u32 = 10;

pub fn contract_graph_parallel<G>(
    graph: &G,
    fraction: f64,
) -> ContractionHierarchy<<G::Edge as EdgeLike>::Distance>
where
    G: GraphLike,
    <G::Edge as EdgeLike>::Distance: Send + Sync,
{
    let working_graph = build_working_graph(graph);

    let start = Instant::now();
    let contraction_hierarchy = contract_working_graph_parallel(working_graph, fraction);
    println!("Contraction took {:?}", start.elapsed());

    contraction_hierarchy
}

fn contract_working_graph_parallel<D: Distance + Send + Sync>(
    mut graph: WorkingGraph<D>,
    fraction: f64,
) -> ContractionHierarchy<D> {
    let mut remaining = (0..graph.num_vertices() as u32)
        .map(VertexId::new)
        .collect::<Vec<_>>();

    let progress = ProgressBar::new(remaining.len() as u64);

    let mut serial_time = Duration::ZERO;
    let mut parallel_time = Duration::ZERO;

    let mut current;

    while !remaining.is_empty() {
        current = Instant::now();
        let (mut next_remaining, candidates) = select_ids(&graph, &remaining);
        serial_time += current.elapsed();

        current = Instant::now();
        let mut candidates_data: Vec<_> = candidates
            .into_par_iter()
            .map(|vertex| {
                let shortcuts = generate_shortcuts(&graph, vertex, MAX_WITNESS_HOPS);
                let edge_difference = edge_difference(&graph, vertex, shortcuts.len());
                (vertex, edge_difference, shortcuts)
            })
            .collect();
        candidates_data.par_sort_unstable_by_key(|(_, edge_difference, _)| *edge_difference);
        parallel_time += current.elapsed();

        current = Instant::now();
        let use_len = clamp(
            (candidates_data.len() as f64 * fraction) as usize,
            1,
            candidates_data.len(),
        );
        for i in use_len..candidates_data.len() {
            next_remaining.push(candidates_data[i].0);
        }

        candidates_data.truncate(use_len);
        println!("candidates len {}", candidates_data.len());

        for (vertex, _, shortcuts) in &candidates_data {
            graph.contract_vertex(*vertex);
            for shortcut in shortcuts {
                graph.add_edge(shortcut);
            }
        }

        remaining = next_remaining;
        serial_time += current.elapsed();
        progress.inc(candidates_data.len() as u64);
    }

    println!("serial time {:?}", serial_time);
    println!("parallel time {:?}", parallel_time);

    progress.finish();

    current = Instant::now();
    let (up_edges, down_edges) = graph.edges();
    let x = ContractionHierarchy::new(
        FastGraph::from_flat(up_edges),
        FastGraph::from_flat(down_edges),
    );
    println!("final creation took {:?}", current.elapsed());

    x
}

fn select_ids<D: Distance>(
    graph: &WorkingGraph<D>,
    candidates: &[VertexId],
) -> (Vec<VertexId>, Vec<VertexId>) {
    if candidates.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut blocked = FxHashSet::default();

    let mut next_remaining = Vec::with_capacity(candidates.len());
    let mut ids = Vec::new();

    for &vertex in candidates {
        if blocked.contains(&vertex) {
            next_remaining.push(vertex);
            continue;
        }

        ids.push(vertex);
        blocked.insert(vertex);

        for edge in graph
            .get_out(vertex)
            .iter()
            .chain(graph.get_in(vertex).iter())
        {
            blocked.insert(edge.head);
        }
    }

    (next_remaining, ids)
}
