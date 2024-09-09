use std::collections::HashMap;

use indicatif::ProgressIterator;

use crate::{
    graphs::{reversible_graph::ReversibleGraph, Graph, Level, TaillessEdge, Vertex, WeightedEdge},
    search::ch::bottom_up::generic::{edge_difference, update_edge_map},
    utility::get_progressbar,
};

pub fn contraction<G, F>(
    mut graph: ReversibleGraph<G>,
    level_to_vertex: &Vec<Level>,
    shortcut_generation: F,
) -> (
    Vec<Vertex>,
    Vec<WeightedEdge>,
    HashMap<(Vertex, Vertex), Vertex>,
)
where
    G: Graph,
    F: Fn(&ReversibleGraph<G>, Vertex) -> HashMap<Vertex, (Vec<TaillessEdge>, Vec<TaillessEdge>)>
        + Send
        + Sync,
{
    let mut edges = graph
        .out_graph()
        .vertices()
        .flat_map(|vertex| graph.out_graph().edges(vertex))
        .collect();

    let mut shortcuts = HashMap::new();

    let number_of_vertices = graph.out_graph().number_of_vertices() as u64;
    let pb = get_progressbar("Contracting", number_of_vertices);

    for &vertex in level_to_vertex.iter().progress_with(pb) {
        let new_and_updated_edges = shortcut_generation(&graph, vertex);
        println!(
            "num edges {}, edge diff {}",
            graph.out_graph().number_of_edges(),
            edge_difference(&graph, &new_and_updated_edges, vertex)
        );
        update_edge_map(&mut edges, &mut shortcuts, vertex, &new_and_updated_edges);
        graph.disconnect(vertex);
        graph.insert_and_update(&new_and_updated_edges);
    }

    (level_to_vertex.clone(), edges, shortcuts)
}
