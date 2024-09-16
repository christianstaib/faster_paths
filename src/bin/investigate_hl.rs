use std::path::PathBuf;

use clap::Parser;
use faster_paths::{
    graphs::Vertex,
    search::{
        ch::contracted_graph::{self, ContractedGraph},
        hl::hub_graph::HubGraph,
        PathFinding,
    },
    utility::{benchmark_distances, benchmark_path, read_bincode_with_spinnner},
};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    num_runs: u32,
}

fn main() {
    let args = Args::parse();

    // Read contracted_graph
    let hub_graph: HubGraph = read_bincode_with_spinnner("hubgraph", &args.hub_graph.as_path());

    println!(
        "average label size {}",
        hub_graph.number_of_entries() as f64 / hub_graph.forward.number_of_vertices() as f64
    );

    println!("shortcuts {}", hub_graph.shortcuts.len());

    let vertices = (0..hub_graph.number_of_vertices()).collect_vec();
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
        benchmark_distances(&hub_graph, &pairs)
    );

    println!(
        "getting random paths takes {:?} on average",
        benchmark_path(&hub_graph, &pairs)
    );
}
