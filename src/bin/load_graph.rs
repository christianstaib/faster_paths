use std::{cmp::Reverse, fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
    },
    search::ch::{
        bottom_up::witness::par_simulate_contraction_witness_search,
        contraction_generic::new_queue_generic,
    },
};
use itertools::Itertools;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    edge_difference: PathBuf,
}

fn main() {
    let args = Args::parse();

    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let mut queue = new_queue_generic(&graph, |graph, vertex| {
        par_simulate_contraction_witness_search(graph, u32::MAX, vertex)
    })
    .into_iter()
    .map(|Reverse((edge_difference, vertex))| (vertex, edge_difference))
    .collect_vec();

    queue.sort_by_key(|&(vertex, _edge_difference)| vertex);

    let edge_difference = queue
        .into_iter()
        .map(|(_vertex, edge_difference)| edge_difference)
        .collect_vec();

    let writer = BufWriter::new(File::create(&args.edge_difference).unwrap());
    serde_json::to_writer(writer, &edge_difference).unwrap();
}
