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
use itertools::Itertools;
use rand::prelude::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,
    #[arg(short, long)]
    edge_differences: PathBuf,
}

fn main() {
    let args = Args::parse();

    let graph: ReversibleGraph<VecVecGraph> =
        ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.graph);

    let hub_graph: HubGraph = {
        let reader = BufReader::new(File::open(&args.hub_graph).unwrap());
        bincode::deserialize_from(reader).unwrap()
    };

    let heuristic = PathfinderHeuristic {
        pathfinder: &hub_graph,
    };

    let mut edge_differences = vec![0; graph.out_graph().number_of_vertices() as usize];

    let mut vertices = graph.out_graph().vertices().collect_vec();
    vertices.shuffle(&mut thread_rng());

    let start = Instant::now();
    for vertex in vertices {
        let new_edges = par_new_edges(&graph, &heuristic, vertex);
        let current_in_edges = graph.in_graph().edges(vertex).len();
        let current_out_edges = graph.out_graph().edges(vertex).len();

        let edge_difference = new_edges - current_in_edges as i32 - current_out_edges as i32;
        println!(
            "vertex {:>9} has edge difference {:>9} (in eddges {:>9}, out edges {:9}). Estimated remaining time {:?}",
            vertex,
            edge_difference,
            current_in_edges,
            current_out_edges,
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
