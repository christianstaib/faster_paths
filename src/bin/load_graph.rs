use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Distance, Graph,
    },
    search::{
        ch::contraction::simulate_contraction,
        collections::{
            dijkstra_data::DijkstraDataVec, vertex_distance_queue::VertexDistanceQueueDaryHeap,
            vertex_expanded_data::VertexExpandedDataBitSet,
        },
        dijkstra::dijktra_one_to_one,
        path::ShortestPathTestCase,
    },
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rand::{thread_rng, Rng};
use rayon::prelude::*;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// TODO
    #[arg(short, long)]
    tests: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Reading test cases");
    let mut reader = BufReader::new(File::open(&args.tests).unwrap());
    let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(&mut reader).unwrap();

    println!("Reading edges");
    let edges = read_edges_from_fmi_file(&args.graph);

    let mut graph = ReversibleGraph::<VecVecGraph>::new();

    println!("Building graph");
    edges.into_iter().for_each(|edge| {
        if edge.weight
            < graph
                .get_weight(&edge.remove_weight())
                .unwrap_or(Distance::MAX)
        {
            graph.set_weight(&edge.remove_weight(), Some(edge.weight));
        }
    });

    println!("set up ch queue");
    (0..graph.out_graph().number_of_vertices())
        .into_par_iter()
        .progress()
        .for_each(|vertex| {
            simulate_contraction(&graph, vertex);
        });

    let mut duration = Duration::ZERO;

    // let graph = graph.out_graph();
    // let graph = &graph;
    // for _ in (0..200).progress().progress() {
    //     let source = thread_rng().gen_range(0..graph.number_of_vertices());
    //     let target = thread_rng().gen_range(0..graph.number_of_vertices());

    //     let mut data = DijkstraDataVec::new(graph);
    //     let mut expanded = VertexExpandedDataBitSet::new(graph);
    //     let mut queue = VertexDistanceQueueDaryHeap::<3>::new();

    //     let start = Instant::now();
    //     dijktra_one_to_one(graph, &mut data, &mut expanded, &mut queue,
    // source, target);     duration += start.elapsed();
    // }
    // println!(
    //     "average duration was {:?}",
    //     duration / (test_cases.len() as u32)
    // );
}
