use std::{path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{large_test_graph, Graph},
    search::{
        alt::landmark::Landmarks,
        ch::{
            contracted_graph::{ch_one_to_one_wrapped, ContractedGraph},
            contraction::{contraction_with_distance_heuristic, contraction_with_witness_search},
        },
        dijkstra::dijkstra_one_to_one_wrapped,
        path::{ShortestPathRequest, ShortestPathTestCase},
    },
};
use indicatif::ProgressIterator;
use itertools::Itertools;
use rand::{seq::IteratorRandom, thread_rng};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
}

fn main() {
    let (graph, test_cases) = large_test_graph();

    let out_graph = graph.out_graph().clone();

    let distance_heuristic = Landmarks::new(
        &graph,
        &(0..graph.out_graph().number_of_vertices()).choose_multiple(&mut thread_rng(), 100),
    );

    println!("Create contracted graph");
    let (level_to_vertex, edges) = contraction_with_distance_heuristic(graph, &distance_heuristic);
    let contracted_graph = ContractedGraph::new(edges, &level_to_vertex);

    let speedup = test_cases
        .iter()
        .progress()
        .map(
            |ShortestPathTestCase {
                 request: ShortestPathRequest { source, target },
                 distance,
             }| {
                let start = Instant::now();
                let ch_distance = ch_one_to_one_wrapped(&contracted_graph, *source, *target);
                let ch_time = start.elapsed().as_secs_f64();

                let start = Instant::now();
                let dijkstra_distance = dijkstra_one_to_one_wrapped(&out_graph, *source, *target);
                let dijkstra_time = start.elapsed().as_secs_f64();

                assert_eq!(distance, &dijkstra_distance);
                assert_eq!(distance, &ch_distance);

                dijkstra_time / ch_time
            },
        )
        .collect_vec();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}
