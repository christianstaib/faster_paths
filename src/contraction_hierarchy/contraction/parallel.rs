use crate::{
    contraction_hierarchy::{
        ContractionEdge,
        contraction::general::{build_working_graph, edge_difference, generate_shortcuts},
        contraction_hierarchy::ContractionHierarchy,
    },
    graph::{DirectionalAdjacencyListGraph, GraphLike, WeightedEdge},
    types::{Distance, Vertex},
};
use num_traits::clamp;
use rayon::prelude::*;
use rustc_hash::FxHashSet;

const MAX_WITNESS_HOPS: u32 = 10;

pub fn contract_graph_parallel<D: Distance>(edges: &[WeightedEdge<D>]) -> ContractionHierarchy<D> {
    let working_graph = build_working_graph(edges.iter());

    let contraction_hierarchy = contract_working_graph_parallel(working_graph, 0.5);

    contraction_hierarchy
}

fn contract_working_graph_parallel<D: Distance>(
    mut graph: DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    fraction: f64,
) -> ContractionHierarchy<D> {
    let mut remaining = (0..graph.num_vertices() as u32)
        .map(Vertex::new)
        .collect::<Vec<_>>();

    while !remaining.is_empty() {
        let (mut next_remaining, candidates) = select_ids(&graph, &remaining);

        let mut candidates_data: Vec<_> = candidates
            .into_par_iter()
            .map(|vertex| {
                let shortcuts = generate_shortcuts(&graph, vertex, MAX_WITNESS_HOPS);
                let edge_difference = edge_difference(&graph, vertex, shortcuts.len());
                (vertex, edge_difference, shortcuts)
            })
            .collect();
        candidates_data.par_sort_unstable_by_key(|(_, edge_difference, _)| *edge_difference);

        let use_len = clamp(
            (candidates_data.len() as f64 * fraction) as usize,
            1,
            candidates_data.len(),
        );
        for i in use_len..candidates_data.len() {
            next_remaining.push(candidates_data[i].0);
        }

        candidates_data.truncate(use_len);

        for (vertex, _, shortcuts) in &candidates_data {
            graph.make_unreachable(*vertex);
            for shortcut in shortcuts {
                graph.add_edge(shortcut);
            }
        }

        remaining = next_remaining;
    }

    let (up_graph, down_graph) = graph.into_csr_graphs();

    let ch = ContractionHierarchy::new(up_graph, down_graph);

    ch
}

fn select_ids<D: Distance>(
    graph: &DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    candidates: &[Vertex],
) -> (Vec<Vertex>, Vec<Vertex>) {
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
            .forward()
            .outgoing_edges(vertex)
            .iter()
            .chain(graph.reverse().outgoing_edges(vertex).iter())
        {
            blocked.insert(edge.head);
        }
    }

    (next_remaining, ids)
}
