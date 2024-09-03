use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
    },
    search::{
        ch::contracted_graph::{self, ContractedGraph},
        hl::hub_graph::HubGraph,
        PathfinderHeuristic,
    },
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
    hub_graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let reader = BufReader::new(File::open(&args.graph).unwrap());
    let graph: ReversibleGraph<VecVecGraph> = bincode::deserialize_from(reader).unwrap();

    let reader = BufReader::new(File::open(&args.contracted_graph).unwrap());
    let contracted_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();

    let reader = BufReader::new(File::open(&args.hub_graph).unwrap());
    let hub_graph: HubGraph = bincode::deserialize_from(reader).unwrap();

    let heuristic = PathfinderHeuristic {
        pathfinder: &hub_graph,
    };

    // Create contracted_graph
    let contracted_graph = ContractedGraph::by_contraction_top_down_with_heuristic(
        &graph,
        contracted_graph.level_to_vertex(),
        &heuristic,
    );

    // // Write contracted_graph to file
    // let writer = BufWriter::new(File::create(&args.contracted_graph).unwrap());
    // bincode::serialize_into(writer, &contracted_graph).unwrap();

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), 1_000);
    let average_duration =
        benchmark_and_test(graph.out_graph(), &tests, &contracted_graph).unwrap();
    println!("Average duration was {:?}", average_duration);
}
