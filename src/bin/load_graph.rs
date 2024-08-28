use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{
        ch::{
            contracted_graph::ContractedGraph, contraction::par_simulate_contraction_witness_search,
        },
        dijkstra::dijkstra_one_to_one_wrapped,
        hl::hub_graph::{get_path_from_overlapp, HubGraph},
    },
};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rand::prelude::*;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("read_edges_from_fmi_file");
    let edges = read_edges_from_fmi_file(&args.graph);

    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let vertices = graph.out_graph().vertices().collect_vec();

    for &vertex in vertices.choose_multiple(&mut thread_rng(), 10) {
        let new_and_updated_edges = par_simulate_contraction_witness_search(&graph, vertex);
    }
}
