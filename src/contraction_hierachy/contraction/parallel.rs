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
type Candidate<D> = (i64, VertexId, Vec<ContractionEdge<D>>);

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
        let batch = select_contraction_batch(&graph, &levels, &terms, fraction);
        let batch_size = batch.len();

        update_terms_for_batch(&mut terms, &graph, &batch);
        apply_contraction_batch(&mut graph, &mut levels, &mut next_level, batch);

        remaining -= batch_size;
        progress.inc(batch_size as u64);
    }

    progress.finish();

    let (up_edges, down_edges) = graph.get_edges();
    build_hierarchy(up_edges, down_edges)
}

fn select_contraction_batch<D: Distance>(
    graph: &WorkingGraph<D>,
    levels: &[usize],
    terms: &[Box<dyn Term<D>>],
    fraction: f64,
) -> Vec<Candidate<D>> {
    let mut candidates = contraction_candidates(graph, levels, terms);
    candidates.sort_unstable_by_key(|(priority, _, _)| *priority);

    select_independent_candidates(graph, levels, &candidates, fraction)
        .into_iter()
        .zip(candidates)
        .filter_map(|(selected, candidate)| selected.then_some(candidate))
        .collect()
}

fn contraction_candidates<D: Distance>(
    graph: &WorkingGraph<D>,
    levels: &[usize],
    terms: &[Box<dyn Term<D>>],
) -> Vec<Candidate<D>> {
    (0..graph.num_vertices() as u32)
        .into_par_iter()
        .map(VertexId::new)
        .filter(|&vertex| is_uncontracted(vertex, levels))
        .map(|vertex| {
            let shortcuts = generate_shortcuts(graph, vertex, MAX_WITNESS_HOPS);
            let priority = priority(graph, vertex, &shortcuts, terms);

            (priority, vertex, shortcuts)
        })
        .collect()
}

fn select_independent_candidates<D: Distance>(
    graph: &WorkingGraph<D>,
    levels: &[usize],
    candidates: &[Candidate<D>],
    fraction: f64,
) -> Vec<bool> {
    let candidate_limit = ((candidates.len() as f64) * fraction).ceil() as usize;
    let candidate_limit = candidate_limit.clamp(1, candidates.len());

    let mut selected = vec![false; candidates.len()];
    let mut blocked = vec![false; graph.num_vertices()];

    for (index, (_, vertex, _)) in candidates.iter().enumerate().take(candidate_limit) {
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

fn update_terms_for_batch<D: Distance>(
    terms: &mut [Box<dyn Term<D>>],
    graph: &WorkingGraph<D>,
    candidates: &[Candidate<D>],
) {
    for (_, vertex, shortcuts) in candidates {
        for term in &mut *terms {
            term.update(graph, *vertex, shortcuts);
        }
    }
}

fn apply_contraction_batch<D: Distance>(
    graph: &mut WorkingGraph<D>,
    levels: &mut [usize],
    next_level: &mut usize,
    candidates: Vec<Candidate<D>>,
) {
    for (_, vertex, _) in &candidates {
        levels[vertex.as_usize()] = *next_level;
        *next_level += 1;
    }

    for (_, vertex, _) in &candidates {
        graph.contract_vertex(*vertex);
    }

    for (_, _, shortcuts) in candidates {
        for shortcut in shortcuts {
            graph.add_edge(shortcut);
        }
    }
}

fn is_uncontracted(vertex: VertexId, levels: &[usize]) -> bool {
    levels[vertex.as_usize()] == usize::MAX
}
