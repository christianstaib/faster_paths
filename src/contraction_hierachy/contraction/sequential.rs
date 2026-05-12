use crate::{
    contraction_hierachy::{
        contraction::{queue::Queue, working_graph::WorkingGraph},
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
    let working_graph = WorkingGraph::new(graph);

    let start = Instant::now();
    let contraction_hierarchy = contract_working_graph_sequential(working_graph);
    println!("Contraction took {:?}", start.elapsed());

    contraction_hierarchy
}

fn contract_working_graph_sequential<D: Distance>(
    mut graph: WorkingGraph<D>,
) -> ContractionHierarchy<D> {
    let mut queue = Queue::new(&graph);
    let progress = ProgressBar::new(graph.num_vertices() as u64);

    while let Some((vertex, shortcuts)) = queue.pop(&graph) {
        graph.contract_vertex(vertex);
        for shortcut in shortcuts {
            graph.add_edge(shortcut);
        }

        progress.inc(1);
    }
    progress.finish();

    let (up_edges, down_edges) = graph.get_edges();
    ContractionHierarchy::new(up_edges, down_edges)
}
