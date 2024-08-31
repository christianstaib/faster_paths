use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph},
    search::{
        ch::bottom_up::heuristic::par_new_edges, hl::hub_graph::HubGraph, PathfinderHeuristic,
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

    let start = Instant::now();
    for vertex in graph.out_graph().vertices() {
        let new_edges = par_new_edges(&graph, &heuristic, vertex);
        let edge_difference = new_edges
            - graph.out_graph().edges(vertex).len() as i32
            - graph.in_graph().edges(vertex).len() as i32;
        println!(
            "vertex {:>9} has edge difference {:>9}. Estimated remaining time {:?}",
            vertex,
            edge_difference,
            start.elapsed() / (vertex + 1)
                * (graph.out_graph().number_of_vertices() - (vertex + 1))
        );

        edge_differences.push(edge_difference);
    }

    {
        let writer = BufWriter::new(File::create(&args.edge_differences).unwrap());
        serde_json::to_writer(writer, &edge_differences).unwrap();
    }
}
