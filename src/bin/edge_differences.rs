use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph},
    search::{
        ch::bottom_up::{generic::edge_difference, heuristic::par_simulate_contraction_heuristic},
        hl::hub_graph::HubGraph,
        PathfinderHeuristic,
    },
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph_bincode: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,
    #[arg(short, long)]
    edge_differences: PathBuf,
}

fn main() {
    let args = Args::parse();

    let graph: ReversibleGraph<VecVecGraph> = {
        let reader = BufReader::new(File::open(&args.graph_bincode).unwrap());
        bincode::deserialize_from(reader).unwrap()
    };

    let hub_graph: HubGraph = {
        let reader = BufReader::new(File::open(&args.hub_graph).unwrap());
        bincode::deserialize_from(reader).unwrap()
    };

    let heuristic = PathfinderHeuristic {
        pathfinder: &hub_graph,
    };

    let mut edge_differences = Vec::new();

    for vertex in graph.out_graph().vertices() {
        let new_and_updated_edges = par_simulate_contraction_heuristic(&graph, &heuristic, vertex);
        let edge_difference = edge_difference(&graph, &new_and_updated_edges, vertex);
        println!(
            "vertex {:>9} has edge difference {:>9}",
            vertex, edge_difference
        );

        edge_differences.push(edge_difference);
    }

    {
        let writer = BufWriter::new(File::create(&args.edge_differences).unwrap());
        serde_json::to_writer(writer, &edge_differences).unwrap();
    }
}
