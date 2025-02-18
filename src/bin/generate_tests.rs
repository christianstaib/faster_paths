use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
    },
    search::PathFinding,
    utility::gen_tests_cases,
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use iter::{IntoParallelIterator, ParallelIterator};
use rayon::*;

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

    let sources_and_targets: Vec<_> = gen_tests_cases(graph.out_graph(), args.number_of_test_cases)
        .into_par_iter()
        .progress()
        .map(|(s, t)| {
            let d = graph
                .shortest_path_distance(s, t)
                .map(|x| x as i64)
                .unwrap_or(-1);

            (s, t, d)
        })
        .collect();

    let mut writer = BufWriter::new(File::create(&args.test_cases).unwrap());
    writeln!(writer, "{}", sources_and_targets.len()).unwrap();
    for &(s, t, d) in sources_and_targets.iter().progress() {
        writeln!(writer, "{} {} {}", s, t, d).unwrap();
    }
}
