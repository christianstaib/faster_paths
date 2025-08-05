use std::path::PathBuf;

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
    },
    reading_pathfinder,
    utility::{benchmark_and_test_path, generate_test_cases},
    FileType,
};

/// Does a single threaded benchmark.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Input file
    #[arg(short, long)]
    file: PathBuf,

    /// Type of the input file
    #[arg(short = 't', long, value_enum, default_value = "fmi")]
    file_type: FileType,

    /// Number of benchmarks to be run.
    #[arg(short, long)]
    number_of_benchmarks: u32,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let pathfinder = reading_pathfinder(&args.file.as_path(), &args.file_type);

    // Benchmark and test correctness
    let tests = generate_test_cases(graph.out_graph(), args.number_of_benchmarks);
    let average_duration = benchmark_and_test_path(graph.out_graph(), &tests, &*pathfinder)
        .expect("Pathfinder ist not correct");
    println!("Average duration was {:?}", average_duration);
}
