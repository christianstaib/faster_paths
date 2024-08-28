use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
    },
    search::ch::contracted_graph::ContractedGraph,
    utility::{benchmark_and_test, generate_test_cases},
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    // Create contracted_graph
    let contracted_graph = ContractedGraph::by_contraction_with_dijkstra_witness_search(&graph);

    // Write contracted_graph to file
    let writer = BufWriter::new(File::create(&args.contracted_graph).unwrap());
    serde_json::to_writer(writer, &contracted_graph).unwrap();

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), 1_000);
    let average_duration =
        benchmark_and_test(graph.out_graph(), &tests, &contracted_graph).unwrap();
    println!("Average duration was {:?}", average_duration);
}
