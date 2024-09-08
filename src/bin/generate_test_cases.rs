use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
    },
    utility::gen_tests_cases,
};

/// Does a single threaded benchmark.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Number of benchmarks to be run.
    #[arg(short, long)]
    number_of_test_cases: u32,

    /// Path of test cases
    #[arg(short, long)]
    test_cases: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let sources_and_targets = gen_tests_cases(graph.out_graph(), args.number_of_test_cases);

    let writer = BufWriter::new(File::create(&args.test_cases).unwrap());
    serde_json::to_writer(writer, &sources_and_targets).unwrap();
}
