use crate::{
    contraction_hierachy::{
        contraction::{
            general::{build_hierarchy, build_working_graph, edge_difference, generate_shortcuts},
            working_graph::WorkingGraph,
        },
        contraction_hierarchy::ContractionHierarchy,
    },
    graph::{EdgeLike, GraphLike},
    types::{Distance, VertexId},
};
use indicatif::{ParallelProgressIterator, ProgressBar};
use rayon::prelude::*;
use std::{cmp::Reverse, collections::BinaryHeap, time::Instant};

const MAX_WITNESS_HOPS: u32 = 10;

pub fn contract_graph_sequential<G>(
    graph: &G,
) -> ContractionHierarchy<<G::Edge as EdgeLike>::Distance>
where
    G: GraphLike,
    <G::Edge as EdgeLike>::Distance: Sync,
{
    let working_graph = build_working_graph(graph);

    let start = Instant::now();
    let contraction_hierarchy = contract_working_graph_sequential(working_graph);
    println!("Contraction took {:?}", start.elapsed());

    contraction_hierarchy
}

fn contract_working_graph_sequential<D: Distance + Sync>(
    mut graph: WorkingGraph<D>,
) -> ContractionHierarchy<D> {
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

    build_hierarchy(graph.contracted_edges(), &levels)
}

/// Initializes the binary heap used during the sequential contraction in parallel.
fn initial_queue<D: Distance + Sync>(
    graph: &WorkingGraph<D>,
) -> BinaryHeap<(Reverse<i64>, VertexId)> {
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
