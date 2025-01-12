use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph, Vertex,
    },
    search::hl::hub_graph::HubGraph,
    utility::{benchmark_and_test_path, generate_test_cases},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    level_to_vertex: PathBuf,
    /// Infile in .fmi format
    #[arg(short = 'u', long)]
    hub_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);
    //
    // Read vertex_to_level
    let reader = BufReader::new(File::open(&args.level_to_vertex).unwrap());
    let level_to_vertex: Vec<Vertex> = serde_json::from_reader(reader).unwrap();

    // Create hub_graph
    let hub_graph = HubGraph::by_brute_force(&graph, &level_to_vertex);
    println!(
        "Average label size forward label {}",
        hub_graph.number_of_entries() as f64
            / graph.out_graph().non_trivial_vertices().len() as f64
    );

    let writer = BufWriter::new(File::create(&args.hub_graph).unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), 1_000);
    let average_duration = benchmark_and_test_path(graph.out_graph(), &tests, &hub_graph).unwrap();
    println!("All correct. Average duration was {:?}", average_duration);
}
