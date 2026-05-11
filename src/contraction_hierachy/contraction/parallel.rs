use crate::{
    contraction_hierachy::{
        ContractionEdge,
        contraction::{
            general::{build_hierarchy, build_working_graph, generate_shortcuts},
            terms::{Term, default_terms, priority},
            working_graph::WorkingGraph,
        },
        contraction_hierarchy::ContractionHierarchy,
    },
    graph::{EdgeLike, GraphLike},
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
{
    let working_graph = build_working_graph(graph);

    let start = Instant::now();
    let contraction_hierarchy = contract_working_graph_parallel(working_graph, fraction);
    println!("Contraction took {:?}", start.elapsed());

    contraction_hierarchy
}

fn contract_working_graph_parallel<D: Distance>(
    mut graph: WorkingGraph<D>,
    fraction: f64,
) -> ContractionHierarchy<D> {
    let mut levels = vec![usize::MAX; graph.num_vertices()];
    let mut terms = default_terms::<D>(graph.num_vertices());
    let mut remaining = graph.num_vertices();
    let mut next_level = 0;
    let progress = ProgressBar::new(remaining as u64);

    while remaining > 0 {
        let mut candidates = contraction_candidates(&graph, &levels, &terms);

        candidates.sort_unstable_by_key(|(vertex, priority, _)| (*priority, *vertex));
        let selected = select_independent_candidates(&graph, &levels, &candidates, fraction);
        debug_assert!(!selected.is_empty());

        let mut selected_candidates = Vec::with_capacity(selected.len());
        for (candidate, selected) in candidates.into_iter().zip(selected) {
            if selected {
                selected_candidates.push(candidate);
            }
        }
        let contracted = selected_candidates.len();

        for (vertex, _, shortcuts) in &selected_candidates {
            for term in &mut terms {
                term.update(&graph, *vertex, shortcuts);
            }
        }

        for (vertex, _, _) in &selected_candidates {
            levels[vertex.as_usize()] = next_level;
            next_level += 1;
        }

        for (vertex, _, _) in &selected_candidates {
            graph.contract_vertex(*vertex);
        }

        for (_, _, shortcuts) in selected_candidates {
            for shortcut in shortcuts {
                if is_uncontracted(shortcut.tail, &levels)
                    && is_uncontracted(shortcut.head, &levels)
                {
                    graph.add_edge(shortcut);
                }
            }
        }

        remaining -= contracted;
        progress.inc(contracted as u64);
    }

    progress.finish();

    debug_assert!(levels.iter().all(|&level| level != usize::MAX));
    build_hierarchy(&graph.get_edges(), &levels)
}

fn contraction_candidates<D: Distance>(
    graph: &WorkingGraph<D>,
    levels: &[usize],
    terms: &[Box<dyn Term<D>>],
) -> Vec<(VertexId, i64, Vec<ContractionEdge<D>>)> {
    (0..graph.num_vertices() as u32)
        .into_par_iter()
        .map(VertexId::new)
        .filter(|&vertex| is_uncontracted(vertex, levels))
        .map(|vertex| {
            let shortcuts = generate_shortcuts(graph, vertex, MAX_WITNESS_HOPS);
            let priority = priority(graph, vertex, &shortcuts, terms);

            (vertex, priority, shortcuts)
        })
        .collect()
}

fn select_independent_candidates<D: Distance>(
    graph: &WorkingGraph<D>,
    levels: &[usize],
    candidates: &[(VertexId, i64, Vec<ContractionEdge<D>>)],
    fraction: f64,
) -> Vec<bool> {
    let candidate_limit = ((candidates.len() as f64) * fraction).ceil() as usize;
    let candidate_limit = candidate_limit.clamp(1, candidates.len());

    let mut selected = vec![false; candidates.len()];
    let mut blocked = vec![false; graph.num_vertices()];

    for (index, (vertex, _, _)) in candidates.iter().enumerate().take(candidate_limit) {
        let vertex_index = vertex.as_usize();
        if blocked[vertex_index] {
            continue;
        }

        selected[index] = true;
        blocked[vertex_index] = true;

        for edge in graph.get_out(*vertex) {
            if is_uncontracted(edge.head, levels) {
                blocked[edge.head.as_usize()] = true;
            }
        }

        for edge in graph.get_in(*vertex) {
            if is_uncontracted(edge.head, levels) {
                blocked[edge.head.as_usize()] = true;
            }
        }
    }

    selected
}

fn is_uncontracted(vertex: VertexId, levels: &[usize]) -> bool {
    levels[vertex.as_usize()] == usize::MAX
}
