use std::{
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

use ahash::{HashMap, HashMapExt};
use indicatif::ProgressBar;
use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    ch::{
        contracted_graph::DirectedContractedGraph,
        contraction_adaptive_simulated::partition_by_levels, Shortcut,
    },
    classical_search::dijkstra::Dijkstra,
    graphs::{
        edge::{DirectedEdge, DirectedWeightedEdge},
        graph_functions::{hitting_set, random_paths},
        hash_graph::HashGraph,
        Graph, VertexId,
    },
};

pub fn contract_non_adaptive(
    mut graph: Box<dyn Graph>,
) -> (DirectedContractedGraph, HashMap<DirectedEdge, VertexId>) {
    println!("copying graph");
    let mut base_graph = HashGraph::from_graph(&*graph);

    let mut shortcuts: HashMap<DirectedEdge, Shortcut> = HashMap::new();
    let mut levels = Vec::new();

    let mut writer = BufWriter::new(File::create("reasons_slow.csv").unwrap());
    writeln!(
            writer,
            "duration_create_shortcuts,duration_add_edges,duration_add_shortcuts,duration_remove_vertex,possible_vertex_shortcuts,vertex_shortcuts,number_of_edges,number_of_shortcuts,number_of_vertices"
        )
        .unwrap();

    println!("starting actual contraction");
    let bar = ProgressBar::new(graph.number_of_vertices() as u64);

    let dijkstra = Dijkstra::new(&*graph);
    let paths = random_paths(10_000, graph.number_of_vertices(), &dijkstra);
    let mut hitting_set = hitting_set(&paths, graph.number_of_vertices()).0;

    let mut not_hitting_set = (0..graph.number_of_vertices())
        .into_iter()
        .filter(|vertex| !hitting_set.contains(&vertex))
        .collect_vec();
    not_hitting_set.shuffle(&mut thread_rng());

    hitting_set.extend(not_hitting_set);

    let mut vertex_buf = Vec::new();
    for &vertex in hitting_set.iter().rev() {
        let start = Instant::now();
        graph.in_edges(vertex).for_each(|in_edge| {
            graph
                .out_edges(vertex)
                .filter_map(|out_edge| {
                    // println!("{:?}", in_edge);
                    // println!("{:?}", out_edge);
                    // println!("");
                    let edge = DirectedWeightedEdge::new(
                        in_edge.tail(),
                        out_edge.head(),
                        in_edge.weight() + out_edge.weight(),
                    )?;
                    let shortcut = Shortcut { edge, vertex };
                    Some(shortcut)
                })
                .for_each(|edge| {
                    vertex_buf.push(edge);
                })
        });

        let vertex_shortcuts: Vec<_> = vertex_buf
            .par_iter()
            // only add edges that are less expensive than currently
            .filter(|shortcut| {
                let current_weight = graph
                    .get_edge_weight(&shortcut.edge.unweighted())
                    .unwrap_or(u32::MAX);
                shortcut.edge.weight() < current_weight
            })
            .cloned()
            .collect();
        vertex_buf.clear();
        let duration_create_shortcuts = start.elapsed();

        let start = Instant::now();
        vertex_shortcuts.iter().for_each(|shortcut| {
            graph.set_edge(&shortcut.edge);
        });
        let duration_add_edges = start.elapsed();

        let possible_shortcuts = graph.in_edges(vertex).len() * graph.out_edges(vertex).len();
        let vertex_shortcuts_len = shortcuts.len();

        let start = Instant::now();
        // insert serial
        for shortcut in vertex_shortcuts {
            shortcuts.insert(shortcut.edge.unweighted(), shortcut);
        }
        let duration_add_shortcuts = start.elapsed();

        let start = Instant::now();
        graph.remove_vertex(vertex);
        let duration_remove_vertex = start.elapsed();

        levels.push(vec![vertex]);
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{}",
            duration_create_shortcuts.as_nanos(),
            duration_add_edges.as_nanos(),
            duration_add_shortcuts.as_nanos(),
            duration_remove_vertex.as_nanos(),
            possible_shortcuts,
            vertex_shortcuts_len,
            graph.number_of_edges(),
            shortcuts.len(),
            graph.number_of_vertices() - levels.len() as u32
        )
        .unwrap();
        writer.flush().unwrap();

        bar.inc(1);
    }
    bar.finish();

    println!("assing shortcuts to base graph");
    let edges = shortcuts
        .values()
        .map(|shortcut| shortcut.edge.clone())
        .collect_vec();
    base_graph.set_edges(&edges);

    println!("creating upward and downward_graph");
    let (upward_graph, downward_graph) = partition_by_levels(&base_graph, &levels);

    println!("generatin shortcut lookup map");
    let shortcuts = shortcuts
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
