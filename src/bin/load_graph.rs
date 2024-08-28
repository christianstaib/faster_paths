use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{
        ch::contracted_graph::ContractedGraph,
        dijkstra::dijkstra_one_to_one_wrapped,
        hl::hub_graph::{get_path_from_overlapp, HubGraph},
    },
};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rand::{thread_rng, Rng};

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

    println!("Create contracted graph");
    let contracted_graph = ContractedGraph::by_contraction_with_dijkstra_witness_search(&graph);

    if graph.out_graph().is_bidirectional() {
        for vertex in graph.out_graph().vertices() {
            let up = contracted_graph.upward_graph().edges(vertex).collect_vec();
            let down = contracted_graph
                .downward_graph()
                .edges(vertex)
                .collect_vec();

            assert_eq!(up, down);
        }
    }
}
