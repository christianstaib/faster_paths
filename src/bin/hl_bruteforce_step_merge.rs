use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph, Vertex,
    },
    search::{
        ch::contracted_graph::vertex_to_level,
        hl::{
            half_hub_graph::{get_hub_label_with_brute_force_wrapped, HalfHubGraph},
            hub_graph::{HubGraph, HubLabelEntry},
        },
        PathFinding,
    },
    utility::{benchmark_and_test_path, generate_test_cases, read_bincode_with_spinnner},
};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    dir: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    level_to_vertex: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);
    println!(
        "graph is bidirectional {}?",
        graph.out_graph().is_bidirectional()
    );

    let reader = BufReader::new(File::open(&args.level_to_vertex).unwrap());
    let level_to_vertex: Vec<Vertex> = serde_json::from_reader(reader).unwrap();
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let mut all_labels = HashMap::new();
    let mut all_shortcuts = HashMap::new();

    for entry in fs::read_dir(args.dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        let (labels, shortcuts): (HashMap<u32, Vec<HubLabelEntry>>, HashMap<(u32, u32), u32>) =
            read_bincode_with_spinnner(path.to_str().unwrap(), path.as_path());

        all_labels.extend(labels);
        all_shortcuts.extend(shortcuts.iter().map(|((t, h), s)| ((*h, *t), *s)));
        all_shortcuts.extend(shortcuts);
    }

    assert_eq!(all_labels.len(), graph.number_of_vertices() as usize);

    let labels = (0..graph.number_of_vertices())
        .map(|vertex| all_labels.get(&vertex).unwrap().clone())
        .collect::<Vec<_>>();

    let hub_graph = HubGraph {
        forward: HalfHubGraph::new(&labels),
        backward: HalfHubGraph::new(&labels),
        shortcuts: all_shortcuts,
        level_to_vertex: level_to_vertex.clone(),
        vertex_to_level,
    };

    let writer = BufWriter::new(File::create(&args.hub_graph).unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), 1_000);
    let average_duration = benchmark_and_test_path(graph.out_graph(), &tests, &hub_graph).unwrap();
    println!("All correct. Average duration was {:?}", average_duration);
}
