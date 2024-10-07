use std::path::PathBuf;

use clap::Parser;
use faster_paths::{
    graphs::Vertex,
    search::{ch::contracted_graph::ContractedGraph, PathFinding},
    utility::{benchmark_distances, benchmark_path, read_bincode_with_spinnner},
};
use itertools::Itertools;
use rand::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    contracted_graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    num_runs: u32,
}

fn main() {
    let args = Args::parse();

    // Read contracted_graph
    let contracted_graph: ContractedGraph =
        read_bincode_with_spinnner("contrated graph", &args.contracted_graph.as_path());

    let vertices = (0..contracted_graph.number_of_vertices()).collect_vec();
    let mut rng = thread_rng();
    let pairs: Vec<(Vertex, Vertex)> = (0..args.num_runs)
        .map(|_| {
            vertices
                .choose_multiple(&mut rng, 2)
                .cloned()
                .collect_tuple()
                .unwrap()
        })
        .collect_vec();

    println!(
        "getting random paths distances takes {:?} on average",
        benchmark_distances(&contracted_graph, &pairs)
    );

    let vertices = (0..contracted_graph.number_of_vertices()).collect_vec();
    let mut rng = thread_rng();
    let pairs: Vec<(Vertex, Vertex)> = (0..args.num_runs)
        .map(|_| {
            vertices
                .choose_multiple(&mut rng, 2)
                .cloned()
                .collect_tuple()
                .unwrap()
        })
        .collect_vec();

    println!(
        "getting random paths takes {:?} on average",
        benchmark_path(&contracted_graph, &pairs)
    );
}
