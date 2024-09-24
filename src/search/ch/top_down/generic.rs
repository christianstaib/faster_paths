use std::{collections::HashMap, time::Instant};

use indicatif::ProgressIterator;
use log::info;

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
        let start = Instant::now();
        let new_and_updated_edges = shortcut_generation(&graph, vertex);
        info!(
            "creating edges took {:?}, will insert {:?} edges",
            start.elapsed(),
            new_and_updated_edges
                .iter()
                .map(|(_, (new, _))| new.len())
                .sum::<usize>()
        );

        let start = Instant::now();
        update_edge_map(&mut edges, &mut shortcuts, vertex, &new_and_updated_edges);
        info!("updating edge map {:?}", start.elapsed());

        let start = Instant::now();
        graph.disconnect(vertex);
        info!("disonecting took {:?}", start.elapsed());

        let start = Instant::now();
        graph.insert_and_update(&new_and_updated_edges);
        info!("insert and update took {:?}", start.elapsed());
    }

    (level_to_vertex.clone(), edges, shortcuts)
}
