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
            contracted_graph::{ch_one_to_one_wrapped, ContractedGraph},
            contraction::contraction_with_distance_heuristic,
        },
        dijkstra::dijkstra_one_to_one_wrapped,
    },
};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng, Rng};

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

    println!("build graph");
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    println!("cloning out graph");
    let out_graph = graph.out_graph().clone();

    println!("getting landmarks");
    let distance_heuristic = Landmarks::new(
        &graph,
        &(0..graph.out_graph().number_of_vertices()).choose_multiple(&mut thread_rng(), 1000),
    );

    println!("Create contracted graph");
    let (level_to_vertex, edges) = contraction_with_distance_heuristic(graph, &distance_heuristic);
    let contracted_graph = ContractedGraph::new(edges, &level_to_vertex);

    let mut rng = thread_rng();
    let speedup = (0..1_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..out_graph.number_of_vertices());
            let target = rng.gen_range(0..out_graph.number_of_vertices());

            let start = Instant::now();
            let ch_distance = ch_one_to_one_wrapped(&contracted_graph, source, target);
            let ch_time = start.elapsed().as_secs_f64();

            let start = Instant::now();
            let dijkstra_distance = dijkstra_one_to_one_wrapped(&out_graph, source, target);
            let dijkstra_time = start.elapsed().as_secs_f64();

            assert_eq!(&ch_distance, &dijkstra_distance);

            dijkstra_time / ch_time
        })
        .collect_vec();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}
