use ahash::{HashMap, HashMapExt};
use indicatif::{ParallelProgressIterator, ProgressStyle};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use crate::{
    classical_search::dijkstra::generate_forward_ch_edges,
    graphs::{
        adjacency_vec_graph::AdjacencyVecGraph, reversible_vec_graph::ReversibleVecGraph, Graph,
    },
};

use super::directed_contracted_graph::DirectedContractedGraph;

pub fn generate_directed_contracted_graph(
    graph: ReversibleVecGraph,
    vertex_to_level_map: Vec<u32>,
) -> DirectedContractedGraph {
    // shuffle vertices for smooth progress bar
    let mut vertices = (0..graph.number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    let mut shortcuts = HashMap::new();

    // upward graph

    let style =
        ProgressStyle::with_template("{wide_bar} {percent_precise}% eta: {eta_precise}").unwrap();
    let upward_shortcuts_and_edges: Vec<_> = vertices
        .par_iter()
        .progress_with_style(style.clone())
        .map(|&vertex| generate_forward_ch_edges(&graph, vertex, &vertex_to_level_map))
        .collect();

    let mut forward_edges = Vec::new();
    for (this_shortcuts, this_edges) in upward_shortcuts_and_edges.into_iter() {
        forward_edges.extend(this_edges);
        shortcuts.extend(
            this_shortcuts
                .iter()
                .map(|(edge, vertex)| (edge.reversed(), *vertex)),
        );
        shortcuts.extend(this_shortcuts);
    }

    let upward_graph = AdjacencyVecGraph::new(&forward_edges, &vertex_to_level_map);

    // downward graph
    let downward_shortcuts_and_edges: Vec<_> = vertices
        .par_iter()
        .progress_with_style(style)
        .map(|&vertex| generate_forward_ch_edges(&graph, vertex, &vertex_to_level_map))
        .collect();

    let mut downward_edges = Vec::new();
    for (this_shortcuts, this_edges) in downward_shortcuts_and_edges.into_iter() {
        downward_edges.extend(this_edges);
        shortcuts.extend(
            this_shortcuts
                .iter()
                .map(|(edge, vertex)| (edge.reversed(), *vertex)),
        );
        shortcuts.extend(this_shortcuts);
    }

    let downward_graph = AdjacencyVecGraph::new(&downward_edges, &vertex_to_level_map);

    // ch graph
    let contracted_graph = DirectedContractedGraph {
        upward_graph,
        downward_graph,
        shortcuts,
        levels: Vec::new(),
    };
    contracted_graph
}
