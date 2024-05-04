use std::{
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use ahash::{HashMap, HashSet};
use dashmap::{DashMap, Map};
use indicatif::{ProgressBar, ProgressIterator};
use itertools::Itertools;
use rayon::prelude::*;

use crate::{
    ch::{
        contracted_graph::DirectedContractedGraph,
        contraction_adaptive_simulated::partition_by_levels, Shortcut,
    },
    graphs::{
        edge::{DirectedEdge, DirectedWeightedEdge},
        graph_functions::all_edges,
        hash_graph::HashGraph,
        path::ShortestPathRequest,
        reversible_hash_graph::ReversibleHashGraph,
        Graph, VertexId,
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};

pub fn contract_adaptive_non_simulated_all_in(
    graph: &dyn Graph,
) -> (DirectedContractedGraph, HashMap<DirectedEdge, VertexId>) {
    println!("copying base graph");
    let mut base_graph = HashGraph::from_graph(&*graph);

    println!("switching graph represenation");
    let mut graph = ReversibleHashGraph::from_edges(&all_edges(&*graph));

    let mut levels = Vec::new();

    let mut writer = BufWriter::new(File::create("reasons_slow.csv").unwrap());
    writeln!(
            writer,
            "duration_create_shortcuts,duration_add_edges,duration_add_shortcuts,duration_remove_vertex,number_of_edges,number_of_shortcuts,number_of_vertices"
        )
        .unwrap();

    println!("starting actual contraction");
    let mut all_shortcuts: DashMap<DirectedEdge, Shortcut> = DashMap::new();

    let mut remaining_vertices: HashSet<VertexId> = (0..graph.number_of_vertices()).collect();
    remaining_vertices.retain(|&vertex| {
        let out_degree = graph.out_edges(vertex).len();
        let in_degree = graph.in_edges(vertex).len();
        out_degree + in_degree > 0
    });
    let bar = ProgressBar::new(remaining_vertices.len() as u64);

    let landmarks = Landmarks::new(0, &graph);

    while let Some(vertex) = get_next_vertex(&graph, &mut remaining_vertices) {
        // generating shortcuts
        let start = Instant::now();
        let shortcuts = generate_all_shortcuts(&graph, &landmarks, vertex, &all_shortcuts);
        let duration_create_shortcuts = start.elapsed();

        // adding shortcuts to graph and all_shortcuts
        let start = Instant::now();
        let edges = shortcuts
            .iter()
            .map(|shortcut| &shortcut.edge)
            .cloned()
            .collect_vec();
        graph.set_edges(&edges);
        let duration_add_edges = start.elapsed();

        let start = Instant::now();
        all_shortcuts.par_extend(
            shortcuts
                .into_par_iter()
                .map(|shortcut| (shortcut.edge.unweighted(), shortcut)),
        );
        let duration_add_shortcuts = start.elapsed();

        // removing graph
        let start = Instant::now();
        graph.remove_vertex(vertex);
        let duration_remove_vertex = start.elapsed();

        levels.push(vec![vertex]);
        writeln!(
            writer,
            "{},{},{},{},{},{},{}",
            duration_create_shortcuts.as_nanos(),
            duration_add_edges.as_nanos(),
            duration_add_shortcuts.as_nanos(),
            duration_remove_vertex.as_nanos(),
            graph.number_of_edges(),
            all_shortcuts.len(),
            remaining_vertices.len()
        )
        .unwrap();
        writer.flush().unwrap();

        bar.inc(1);
    }
    bar.finish();

    let all_shortcuts: HashMap<DirectedEdge, Shortcut> = all_shortcuts.into_iter().collect();
    println!("writing base_grap and shortcuts to file");
    let writer = BufWriter::new(File::create("all_in.bincode").unwrap());
    bincode::serialize_into(writer, &(&base_graph, &all_shortcuts)).unwrap();

    println!("Building edge vec");
    let edges = all_shortcuts
        .values()
        .progress()
        .map(|shortcut| shortcut.edge.clone())
        .collect_vec();

    println!("Adding shortcuts to base graph");
    base_graph.set_edges(&edges);

    println!("creating upward and downward_graph");
    let (upward_graph, downward_graph) = partition_by_levels(&base_graph, &levels);

    println!("generatin shortcut lookup map");
    let shortcuts = all_shortcuts
        .values()
        .map(|shortcut| (shortcut.edge.unweighted(), shortcut.vertex))
        .collect();

    let directed_contracted_graph = DirectedContractedGraph {
        upward_graph,
        downward_graph,
        levels,
    };

    (directed_contracted_graph, shortcuts)
}

fn generate_all_shortcuts(
    graph: &dyn Graph,
    heuristic: &dyn Heuristic,
    vertex: u32,
    all_shortcuts: &DashMap<DirectedEdge, Shortcut>,
) -> Vec<Shortcut> {
    let in_edges = graph.in_edges(vertex).collect_vec();
    let out_edges = graph.out_edges(vertex).collect_vec();

    let shortcuts: Vec<_> = in_edges
        .par_iter()
        .map(|in_edge| {
            out_edges
                .iter()
                .filter(|out_edge| in_edge.tail() != out_edge.head())
                .filter_map(|out_edge| {
                    let edge = DirectedWeightedEdge::new(
                        in_edge.tail(),
                        out_edge.head(),
                        in_edge.weight() + out_edge.weight(),
                    )
                    .unwrap();

                    // edge is cheaper than upper bound heuristic
                    let request =
                        ShortestPathRequest::new(in_edge.tail(), out_edge.head()).unwrap();
                    if let Some(upper_bound) = heuristic.upper_bound(&request) {
                        if edge.weight() >= upper_bound {
                            return None;
                        }
                    }

                    // check if new shortcut is cheaper than current shortcut (if it exists)
                    if let Some(current_shortcut) = all_shortcuts.get(&edge.unweighted()) {
                        let current_shortcut_weight = current_shortcut.edge.weight();
                        if edge.weight() >= current_shortcut_weight {
                            return None;
                        }
                    }

                    // check if new edge is cheaper than current edge (if it exists)
                    if let Some(current_weight) = graph.get_edge_weight(&edge.unweighted()) {
                        if edge.weight() >= current_weight {
                            return None;
                        }
                    }

                    let shortcut = Shortcut { edge, vertex };

                    Some(shortcut)
                })
                .collect_vec()
        })
        .flatten()
        .collect();
    shortcuts
}

fn get_next_vertex(
    graph: &dyn Graph,
    remaining_vertices: &mut HashSet<VertexId>,
) -> Option<VertexId> {
    let min_vertex = *remaining_vertices.par_iter().min_by_key(|&&vertex| {
        (graph.in_edges(vertex).len() as i32 * graph.out_edges(vertex).len() as i32)
        // - (graph.in_edges(vertex).len() as i32)
        // - graph.out_edges(vertex).len() as i32
    })?;
    remaining_vertices.remove(&min_vertex);
    Some(min_vertex)
}
