use std::{collections::HashSet, fs::File, io::BufWriter, ops::Sub, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{
        ch::{
            brute_force::{brute_force_contracted_graph, get_ch_edges, get_ch_edges_wrapped},
            contracted_graph::{ch_one_to_one_wrapped, ContractedGraph},
            contraction::contraction_with_witness_search,
        },
        collections::{
            dijkstra_data::{DijkstraData, DijkstraDataVec},
            vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueBinaryHeap},
            vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
        },
        dijkstra::{dijkstra_one_to_one, dijkstra_one_to_one_wrapped},
    },
};
use indicatif::ProgressIterator;
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

    println!("build graph");
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    println!("cloning out graph");
    let cloned_graph = graph.clone();

    println!("Create contracted graph");
    let (level_to_vertex, edges) = contraction_with_witness_search(graph);
    let contracted_graph = ContractedGraph::new(edges, &level_to_vertex);

    let other_contracted_graph =
        brute_force_contracted_graph(&cloned_graph, &contracted_graph.level_to_vertex);

    let mut rng = thread_rng();
    let speedup = (0..100_000)
        .progress()
        .map(|_| {
            let source = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());
            let target = rng.gen_range(0..cloned_graph.out_graph().number_of_vertices());

            let start = Instant::now();
            let ch_distance = ch_one_to_one_wrapped(&other_contracted_graph, source, target);
            let ch_time = start.elapsed().as_secs_f64();

            let start = Instant::now();
            let dijkstra_distance =
                dijkstra_one_to_one_wrapped(cloned_graph.out_graph(), source, target)
                    .map(|path| path.distance);
            let dijkstra_time = start.elapsed().as_secs_f64();

            assert_eq!(&ch_distance, &dijkstra_distance);

            dijkstra_time / ch_time
        })
        .collect::<Vec<_>>();

    println!(
        "average speedups {:?}",
        speedup.iter().sum::<f64>() / speedup.len() as f64
    );
}
