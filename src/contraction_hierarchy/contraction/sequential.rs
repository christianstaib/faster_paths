use crate::{
    contraction_hierarchy::{
        ContractionEdge,
        contraction::{
            general::{build_working_graph, generate_shortcuts},
            queue::Queue,
        },
        contraction_hierarchy::ContractionHierarchy,
    },
    graph::{DirectionalAdjacencyListGraph, WeightedEdge},
    types::{Distance, Vertex},
};

pub fn contract_graph_sequential<D: Distance>(
    edges: &Vec<WeightedEdge<D>>,
) -> ContractionHierarchy<D> {
    let working_graph = build_working_graph(edges.iter());

    contract_working_graph_sequential(working_graph)
}

fn contract_working_graph_sequential<D: Distance>(
    mut graph: DirectionalAdjacencyListGraph<ContractionEdge<D>>,
) -> ContractionHierarchy<D> {
    let mut queue = Queue::new(&graph);

    while let Some((vertex, shortcuts)) = queue.pop(&graph) {
        graph.make_unreachable(vertex);
        for shortcut in &shortcuts {
            graph.add_edge(shortcut);
        }
    }

    let (up_graph, down_graph) = graph.into_csr_graphs();
    ContractionHierarchy::new(up_graph, down_graph)
}

pub fn contract_working_graph_sequential_with_order<D: Distance>(
    mut graph: DirectionalAdjacencyListGraph<ContractionEdge<D>>,
    order: &Vec<Vertex>,
) -> ContractionHierarchy<D> {
    for &vertex in order.iter().rev() {
        let shortcuts = generate_shortcuts(&graph, vertex, 10);
        graph.make_unreachable(vertex);
        for shortcut in &shortcuts {
            graph.add_edge(shortcut);
        }
    }

    let (up_graph, down_graph) = graph.into_csr_graphs();
    ContractionHierarchy::new(up_graph, down_graph)
}
