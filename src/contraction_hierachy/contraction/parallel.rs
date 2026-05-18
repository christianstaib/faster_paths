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
use rayon::prelude::*;
use std::time::Instant;

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
    // let mut levels = vec![usize::MAX; graph.num_vertices()];
    let mut remaining = (0..graph.num_vertices() as u32)
        .map(VertexId::new)
        .collect::<Vec<_>>();
    let mut blocked = vec![0u32; graph.num_vertices()];
    let mut block_token = 1;
    let progress = ProgressBar::new(remaining.len() as u64);

    while !remaining.is_empty() {
        sort_vertices_by_degree(&graph, &mut remaining);
        let (next_remaining, ids) =
            select_ids(&graph, &remaining, fraction, &mut blocked, block_token);
        debug_assert!(!ids.is_empty());

        let mut selected_candidates = build_shortcuts_for_vertices(&graph, ids);
        debug_assert!(!selected_candidates.is_empty());

        selected_candidates
            .sort_unstable_by_key(|(vertex, edge_difference, _)| (*edge_difference, *vertex));
        let contracted = selected_candidates.len();

        for (vertex, _, _) in &selected_candidates {
            graph.contract_vertex(*vertex);
        }

        for (_, _, shortcuts) in selected_candidates {
            for shortcut in shortcuts {
                graph.add_edge(shortcut);
            }
        }

        remaining = next_remaining;
        block_token = block_token.wrapping_add(1);
        if block_token == 0 {
            blocked.fill(0);
            block_token = 1;
        }
        progress.inc(contracted as u64);
    }

    progress.finish();

    let (up_edges, down_edges) = graph.edges();
    ContractionHierarchy::new(
        FastGraph::from_flat(up_edges),
        FastGraph::from_flat(down_edges),
    )
}

fn sort_vertices_by_degree<D: Distance + Sync>(graph: &WorkingGraph<D>, vertices: &mut [VertexId]) {
    vertices.par_sort_unstable_by_key(|&vertex| {
        (
            graph.get_out(vertex).len() + graph.get_in(vertex).len(),
            vertex,
        )
    });
}

fn select_ids<D: Distance>(
    graph: &WorkingGraph<D>,
    candidates: &[VertexId],
    fraction: f64,
    blocked: &mut [u32],
    block_token: u32,
) -> (Vec<VertexId>, Vec<VertexId>) {
    if candidates.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let candidate_limit = ((candidates.len() as f64) * fraction).ceil() as usize;
    let candidate_limit = candidate_limit.clamp(1, candidates.len());

    let mut next_remaining = Vec::with_capacity(candidates.len());
    let mut ids = Vec::new();

    for (index, &vertex) in candidates.iter().enumerate() {
        if index >= candidate_limit || blocked[vertex.as_usize()] == block_token {
            next_remaining.push(vertex);
            continue;
        }

        ids.push(vertex);
        blocked[vertex.as_usize()] = block_token;

        for edge in graph.get_out(vertex) {
            blocked[edge.head.as_usize()] = block_token;
        }

        for edge in graph.get_in(vertex) {
            blocked[edge.head.as_usize()] = block_token;
        }
    }

    (next_remaining, ids)
}

fn build_shortcuts_for_vertices<D: Distance + Send + Sync>(
    graph: &WorkingGraph<D>,
    vertices: Vec<VertexId>,
) -> Vec<(VertexId, i64, Vec<ContractionEdge<D>>)> {
    vertices
        .into_par_iter()
        .map(|vertex| {
            let shortcuts = generate_shortcuts(graph, vertex, MAX_WITNESS_HOPS);
            let edge_difference = edge_difference(graph, vertex, shortcuts.len());

            (vertex, edge_difference, shortcuts)
        })
        .collect()
}
