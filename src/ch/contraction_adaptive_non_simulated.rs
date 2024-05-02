use std::{
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use ahash::{HashMap, HashMapExt, HashSet};
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

    let mut all_shortcuts: HashMap<DirectedEdge, Shortcut> = HashMap::new();
    let mut levels = Vec::new();

    let mut remaining_vertices: HashSet<VertexId> = (0..graph.number_of_vertices()).collect();
    remaining_vertices.retain(|&vertex| {
        let out_degree = graph.out_edges(vertex).len();
        let in_degree = graph.in_edges(vertex).len();
        out_degree + in_degree > 0
    });

    let mut writer = BufWriter::new(File::create("reasons_slow.csv").unwrap());
    writeln!(
            writer,
            "duration_create_shortcuts,duration_add_edges,duration_remove_vertex,number_of_edges,number_of_shortcuts,number_of_vertices"
        )
        .unwrap();

    println!("starting actual contraction");
    let bar = ProgressBar::new(remaining_vertices.len() as u64);

    let landmarks = Landmarks::new(25, &graph);

    while let Some(vertex) = get_next_vertex(&graph, &mut remaining_vertices) {
        // generating shortcuts
        let start = Instant::now();
        let shortcuts: Vec<_> = graph
            .in_edges(vertex)
            .par_bridge()
            .map(|in_edge| {
                graph
                    .out_edges(vertex)
                    .filter(|out_edge| in_edge.tail() != out_edge.head())
                    .map(|out_edge| {
                        let edge = DirectedWeightedEdge::new(
                            in_edge.tail(),
                            out_edge.head(),
                            in_edge.weight() + out_edge.weight(),
                        )
                        .unwrap();
                        Shortcut { edge, vertex }
                    })
                    .collect_vec()
            })
            .flatten()
            // only add edges that are less expensive than currently
            .filter(|shortcut| {
                let edge = shortcut.edge.unweighted();
                let current_weight = graph.get_edge_weight(&edge).unwrap_or(u32::MAX);
                shortcut.edge.weight() < current_weight
            })
            // // only add edges that are less expensive than currently
            // .filter(|shortcut| {
            //     let request =
            //         ShortestPathRequest::new(shortcut.edge.tail(),
            // shortcut.edge.head()).unwrap();     let upper_bound =
            // landmarks.upper_bound(&request).unwrap_or(u32::MAX);     shortcut.edge.
            // weight() < upper_bound })
            .collect();
        let duration_create_shortcuts = start.elapsed();

        // adding shortcuts to graph and all_shortcuts
        let start = Instant::now();
        shortcuts.into_iter().for_each(|shortcut| {
            graph.set_edge(&shortcut.edge);
            all_shortcuts.insert(shortcut.edge.unweighted(), shortcut);
        });
        let duration_add_edges = start.elapsed();

        // removing graph
        let start = Instant::now();
        graph.remove_vertex(vertex);
        let duration_remove_vertex = start.elapsed();

        levels.push(vec![vertex]);
        writeln!(
            writer,
            "{},{},{},{},{},{}",
            duration_create_shortcuts.as_nanos(),
            duration_add_edges.as_nanos(),
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

fn get_next_vertex(
    graph: &dyn Graph,
    remaining_vertices: &mut HashSet<VertexId>,
) -> Option<VertexId> {
    let min_vertex = *remaining_vertices.par_iter().min_by_key(|&&vertex| {
        (graph.in_edges(vertex).len() as i32 * graph.out_edges(vertex).len() as i32)
            - (graph.in_edges(vertex).len() as i32)
            - graph.out_edges(vertex).len() as i32
    })?;
    remaining_vertices.remove(&min_vertex);
    Some(min_vertex)
}
