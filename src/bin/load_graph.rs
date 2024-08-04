use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    graphs::{read_edges_from_fmi_file, vec_vec_graph::VecVecGraph, Graph},
    search::{
        dijkstra::{dijktra_single_pair, dijktra_single_source},
        dijkstra_data::{DijkstraData, DijkstraDataVec},
        path::ShortestPathTestCase,
        vertex_distance_queue::VertexDistanceQueueRadixHeap,
        vertex_expanded_data::VertexExpandedDataVec,
    },
};

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
    let mut test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(&mut reader).unwrap();

    println!("Loading graph");
    let mut graph = VecVecGraph::default();
    read_edges_from_fmi_file(&args.graph)
        .iter()
        .for_each(|edge| graph.set_weight(&edge.remove_weight(), Some(edge.weight)));

    println!(
        "graph has {} vertices and {} edges",
        graph.number_of_vertices(),
        graph.number_of_edges()
    );

    let mut duration = Duration::ZERO;

    for (test_index, test) in test_cases.iter().enumerate() {
        let source = test.request.source;
        let target = test.request.target;

        let mut data = DijkstraDataVec::new(&graph);
        let mut expanded = VertexExpandedDataVec::new(&graph);
        let mut queue = VertexDistanceQueueRadixHeap::new();
        let start = Instant::now();
        dijktra_single_pair(&graph, &mut data, &mut expanded, &mut queue, source, target);
        duration += start.elapsed();
        println!(
            "distance from {:>12} to {:>12} is {:>12?} and should be {:>12?}. took {:>12?}. avg {:>12?}",
            source,
            target,
            data.get_distance(target),
            test.weight,
            start.elapsed(),
            duration / (test_index as u32 + 1)
        );
    }
}
