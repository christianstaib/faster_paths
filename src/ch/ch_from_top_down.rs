use ahash::{HashMap, HashMapExt};
use indicatif::{ParallelProgressIterator, ProgressStyle};
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;

use crate::{
    classical_search::dijkstra::top_down_ch,
    graphs::{
        adjacency_vec_graph::AdjacencyVecGraph, reversible_vec_graph::ReversibleVecGraph, Graph,
    },
};

use super::directed_contracted_graph::DirectedContractedGraph;

pub fn generate_directed_contracted_graph(
    graph: ReversibleVecGraph,
    vertex_to_level_map: Vec<u32>,
) -> DirectedContractedGraph {
    let mut vertices = (0..graph.number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    let style =
        ProgressStyle::with_template("{wide_bar} {eta_precise}/{duration_precise}").unwrap();

    let forward_shortcuts_and_edges: Vec<_> = vertices
        .into_par_iter()
        .progress_with_style(style)
        .map(|vertex| top_down_ch(&graph, vertex, &vertex_to_level_map))
        .collect();

    let mut forward_edges = Vec::new();
    let mut forward_shortcuts = HashMap::new();
    for (shortcuts, edges) in forward_shortcuts_and_edges.into_iter() {
        forward_edges.extend(edges);
        forward_shortcuts.extend(
            shortcuts
                .iter()
                .map(|(edge, vertex)| (edge.reversed(), *vertex)),
        );
        forward_shortcuts.extend(shortcuts);
    }

    let upward_graph = AdjacencyVecGraph::new(&forward_edges, &vertex_to_level_map);
    let downward_graph = upward_graph.clone();
    let contracted_graph = DirectedContractedGraph {
        upward_graph,
        downward_graph,
        shortcuts: forward_shortcuts,
        levels: Vec::new(),
    };
    contracted_graph
}
