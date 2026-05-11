use crate::{
    contraction_hierachy::{
        contraction::{
            general::{build_hierarchy, build_working_graph},
            queue::Queue,
            working_graph::WorkingGraph,
        },
        contraction_hierarchy::ContractionHierarchy,
    },
    graph::{EdgeLike, GraphLike},
    types::Distance,
};
use indicatif::ProgressBar;
use std::time::Instant;

pub fn contract_graph_sequential<G>(
    graph: &G,
) -> ContractionHierarchy<<G::Edge as EdgeLike>::Distance>
where
    G: GraphLike,
{
    let working_graph = build_working_graph(graph);

    let start = Instant::now();
    let contraction_hierarchy = contract_working_graph_sequential(working_graph);
    println!("Contraction took {:?}", start.elapsed());

    contraction_hierarchy
}

fn contract_working_graph_sequential<D: Distance>(
    mut graph: WorkingGraph<D>,
) -> ContractionHierarchy<D> {
    let mut levels = vec![0; graph.num_vertices()];
    let mut queue = Queue::new(&graph);
    let progress = ProgressBar::new(queue.len() as u64);

    let mut next_level = 0;
    while let Some((vertex, shortcuts)) = queue.pop(&graph) {
        graph.contract_vertex(vertex);
        for shortcut in shortcuts {
            graph.add_edge(shortcut);
        }

        levels[vertex.as_usize()] = next_level;
        next_level += 1;
        progress.inc(1);
    }
    progress.finish();

    build_hierarchy(&graph.get_edges(), &levels)
}
