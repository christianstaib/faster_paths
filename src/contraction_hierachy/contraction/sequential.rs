use crate::{
    contraction_hierachy::{
        ContractionEdge,
        contraction::{general::build_working_graph, queue::Queue},
        contraction_hierarchy::ContractionHierarchy,
    },
    graph::{DirectionalAdjacencyListGraph, EdgeLike, GraphLike},
    types::Distance,
};
use indicatif::ProgressBar;
use std::time::Instant;

pub fn contract_graph_sequential<G>(
    graph: &G,
) -> ContractionHierarchy<<G::Edge as EdgeLike>::Weight>
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
    mut graph: DirectionalAdjacencyListGraph<ContractionEdge<D>>,
) -> ContractionHierarchy<D> {
    let mut queue = Queue::new(&graph);
    let progress = ProgressBar::new(graph.num_vertices() as u64);

    while let Some((vertex, shortcuts)) = queue.pop(&graph) {
        graph.make_unreachable(vertex);
        for shortcut in &shortcuts {
            graph.add_edge(shortcut);
        }

        progress.inc(1);
    }
    progress.finish();

    let (up_graph, down_graph) = graph.into_csr_graphs();
    ContractionHierarchy::new(up_graph, down_graph)
}
