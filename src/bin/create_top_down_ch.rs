use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use ahash::{HashMap, HashMapExt};
use clap::Parser;
use faster_paths::{
    ch::directed_contracted_graph::DirectedContractedGraph,
    classical_search::dijkstra::top_down_ch,
    graphs::{
        adjacency_vec_graph::AdjacencyVecGraph, graph_factory::GraphFactory,
        graph_functions::generate_vertex_to_level_map, path::Path, Graph,
    },
    hl::hl_from_top_down::{generate_forward_label, generate_reverse_label},
};
use indicatif::{ParallelProgressIterator, ProgressStyle};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// Creates a hub graph top down.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    paths: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("loading paths");
    let reader = BufReader::new(File::open(&args.paths).unwrap());
    let paths: Vec<Path> = serde_json::from_reader(reader).unwrap();

    println!("loading graph");
    let graph = GraphFactory::from_file(&args.graph);

    let vertex_to_level_map = generate_vertex_to_level_map(paths, graph.number_of_vertices);

    let mut level_to_verticies_map = vec![Vec::new(); vertex_to_level_map.len()];
    for (vertex, &level) in vertex_to_level_map.iter().enumerate() {
        level_to_verticies_map[level as usize].push(vertex as u32);
    }

    let mut vertices = (0..graph.number_of_vertices()).collect_vec();
    vertices.shuffle(&mut thread_rng());

    let style =
        ProgressStyle::with_template("{wide_bar} {eta_precise}/{duration_precise}").unwrap();

    let start = Instant::now();
    let forward_shortcuts: Vec<_> = vertices
        .into_par_iter()
        .progress_with_style(style)
        .map(|vertex| {
            let shortcuts = top_down_ch(&graph, vertex, &vertex_to_level_map);
            let mut ch_neighbors_ch = Vec::new();
            for shortcut in shortcuts.iter() {
                ch_neighbors_ch.push(shortcut.edge.head());
            }

            shortcuts
        })
        .flatten()
        .collect();
    println!("took {:?}", start.elapsed());
    println!(
        "there are {} ch edges and {} normal ones",
        forward_shortcuts.len(),
        graph.number_of_edges()
    );

    let upward_edges: Vec<_> = forward_shortcuts
        .iter()
        .map(|shortcut| shortcut.edge.clone())
        .collect();
    let upward_graph = AdjacencyVecGraph::new(&upward_edges, &vertex_to_level_map);
    let downward_graph = upward_graph.clone();
    let contracted_graph = DirectedContractedGraph {
        upward_graph,
        downward_graph,
        shortcuts: HashMap::new(),
        levels: Vec::new(),
    };

    println!("Writing contracted graph to file");
    let writer = BufWriter::new(File::create(args.contracted_graph).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();
}
