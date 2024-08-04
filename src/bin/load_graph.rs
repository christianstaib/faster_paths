use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    graphs::{read_edges_from_fmi_file, vec_vec_graph::VecVecGraph, Graph, Weight},
    search::{
        dijkstra::dijktra_single_pair,
        dijkstra_data::{DijkstraData, DijkstraDataVec},
        path::ShortestPathTestCase,
        vertex_distance_queue::{VertexDistanceQueue, VertexDistanceQueueRadixHeap},
        vertex_expanded_data::{VertexExpandedData, VertexExpandedDataBitSet},
    },
};
use indicatif::ProgressIterator;

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

    println!("Loading graph");
    let mut graph = VecVecGraph::default();
    read_edges_from_fmi_file(&args.graph)
        .iter()
        .for_each(|edge| {
            if edge.weight
                < graph
                    .get_weight(&edge.remove_weight())
                    .unwrap_or(Weight::MAX)
            {
                graph.set_weight(&edge.remove_weight(), Some(edge.weight))
            }
        });

    println!(
        "graph has {} vertices and {} edges",
        graph.number_of_vertices(),
        graph.number_of_edges()
    );

    let mut duration = Duration::ZERO;

    let mut data = DijkstraDataVec::new(&graph);
    let mut expanded = VertexExpandedDataBitSet::new(&graph);
    let mut queue = VertexDistanceQueueRadixHeap::new();
    for test in test_cases.iter().progress() {
        data.clear();
        expanded.clear();
        queue.clear();

        let source = test.request.source;
        let target = test.request.target;

        let start = Instant::now();
        dijktra_single_pair(&graph, &mut data, &mut expanded, &mut queue, source, target);
        duration += start.elapsed();
        if data.get_distance(target) != test.weight {
            println!(
                "{} to {} is {:?} but should be {:?}",
                source,
                target,
                data.get_distance(target),
                test.weight,
            );
        }
    }
    println!(
        "average duration was {:?}",
        duration / (test_cases.len() as u32)
    );
}
