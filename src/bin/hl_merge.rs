use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
    },
    search::hl::hub_graph::HubGraph,
    utility::{benchmark_and_test, generate_test_cases},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    // Read contracted_graph
    let reader = BufReader::new(File::open(&args.contracted_graph).unwrap());
    let contracted_graph = serde_json::from_reader(reader).unwrap();

    // Create hub_graph
    let hub_graph = HubGraph::by_merging(&contracted_graph);
    println!("Average label size is {}", hub_graph.average_label_size());

    // Write hub_graph to file
    let writer = BufWriter::new(File::create(&args.contracted_graph).unwrap());
    serde_json::to_writer(writer, &contracted_graph).unwrap();

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), 1_000);
    let average_duration = benchmark_and_test(graph.out_graph(), &tests, &hub_graph).unwrap();
    println!("Average duration was {:?}", average_duration);
}
