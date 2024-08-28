use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{
        alt::landmark::Landmarks,
        ch::{
            contracted_graph::ContractedGraph,
            contraction::{edge_difference, par_simulate_contraction_witness_search},
            probabilistic_contraction::par_simulate_contraction_distance_heuristic,
        },
        dijkstra::dijkstra_one_to_one_wrapped,
        hl::hub_graph::{get_path_from_overlapp, HubGraph},
        TrivialHeuristic,
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

    // let heuristic = TrivialHeuristic {};
    let heuristic = Landmarks::random(&graph, rayon::current_num_threads() as u32 * 2);

    let vertices = graph.out_graph().vertices().collect_vec();

    for &vertex in vertices.choose_multiple(&mut thread_rng(), 100) {
        // let new_and_updated_edges_witness =
        //     par_simulate_contraction_witness_search(&graph, u32::MAX, vertex);
        // let edge_differece_witness =
        //     edge_difference(&graph, &new_and_updated_edges_witness, vertex);
        // println!(
        //     "edge_difference for vertex {} is {} (outgoing edges {})",
        //     vertex,
        //     edge_differece_witness,
        //     graph.out_graph().edges(vertex).len()
        // );

        let new_and_updated_edges_heuristic =
            par_simulate_contraction_distance_heuristic(&graph, &heuristic, vertex);
        let edge_differece_heuristic =
            edge_difference(&graph, &new_and_updated_edges_heuristic, vertex);
        println!(
            "edge_difference for vertex {} is {} (outgoing edges {})",
            vertex,
            edge_differece_heuristic,
            graph.out_graph().edges(vertex).len()
        );
    }
}
