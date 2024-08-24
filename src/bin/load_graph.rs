use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Edge, Graph,
    },
    search::{
        ch::contracted_graph::{ch_one_to_one_wrapped, ContractedGraph},
        dijkstra::dijkstra_one_to_one_wrapped,
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

    println!("brute_force");

    let mut rng = thread_rng();
    let speedup = (0..100_000)
        .progress()
        .map(|_| {
            let source = 123; //rng.gen_range(0..graph.out_graph().number_of_vertices());
            let target = 2345; //rng.gen_range(0..graph.out_graph().number_of_vertices());

            let start = Instant::now();
            let hl_path = ch_one_to_one_wrapped(&contracted_graph, source, target);
            let ch_time = start.elapsed().as_secs_f64();

            let hl_distance = hl_path.as_ref().map(|path| path.distance);
            if let Some(hl_path) = &hl_path {
                let mut distance = 0;
                for (&tail, &head) in hl_path.vertices.iter().tuple_windows() {
                    distance += graph.out_graph().get_weight(&Edge { tail, head }).unwrap();
                }
                assert_eq!(distance, hl_distance.unwrap());
            }

            let start = Instant::now();
            let dijkstra_distance = dijkstra_one_to_one_wrapped(graph.out_graph(), source, target)
                .map(|path| path.distance);
            let dijkstra_time = start.elapsed().as_secs_f64();

            assert_eq!(&hl_distance, &dijkstra_distance);

            dijkstra_time / ch_time
        })
        .collect::<Vec<_>>();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}
