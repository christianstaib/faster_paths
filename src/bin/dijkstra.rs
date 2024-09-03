use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph},
    utility::{benchmark, gen_tests_cases},
};

/// Reading a .bincode file is way faster than a .fmi file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short = 'b', long)]
    graph_bincode: PathBuf,
}

fn main() {
    let args = Args::parse();

    let writer = BufReader::new(File::open(&args.graph_bincode).unwrap());
    let graph: ReversibleGraph<VecVecGraph> = bincode::deserialize_from(writer).unwrap();

    let m = 100;
    println!("Value over {} sequential searches", m);
    let sources_and_targets = gen_tests_cases(graph.out_graph(), m);
    let avg_dijkstra_duration = benchmark(graph.out_graph(), &sources_and_targets);
    println!("Average dijkstra duration is {:?}", avg_dijkstra_duration);
}
