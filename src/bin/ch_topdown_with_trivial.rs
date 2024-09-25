use std::path::PathBuf;

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_hash_graph::VecHashGraph,
        vec_vec_graph::VecVecGraph, Graph, Vertex,
    },
    search::{ch::contracted_graph::ContractedGraph, TrivialHeuristic},
    utility::{
        benchmark_and_test_path, generate_test_cases, read_bincode_with_spinnner,
        read_json_with_spinnner, write_bincode_with_spinnner,
    },
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
    level_to_vertex: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();
    env_logger::init();

    // Build graph
    let graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("graph", &args.graph.as_path());
    let graph: ReversibleGraph<VecHashGraph> =
        ReversibleGraph::from_edges(&graph.out_graph().all_edges());

    let level_to_vertex: Vec<Vertex> =
        read_json_with_spinnner("level to vertex", &args.level_to_vertex.as_path());

    // Create contracted_graph
    let contracted_graph = ContractedGraph::by_contraction_top_down_with_heuristic(
        &graph,
        &level_to_vertex,
        &TrivialHeuristic {},
    );

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), 1_000);
    let average_duration =
        benchmark_and_test_path(graph.out_graph(), &tests, &contracted_graph).unwrap();
    println!("Average duration was {:?}", average_duration);

    // Write contracted_graph to file
    write_bincode_with_spinnner(
        "contracted_graph",
        &args.contracted_graph.as_path(),
        &contracted_graph,
    );
}
