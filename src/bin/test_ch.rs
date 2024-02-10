use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use indicatif::ProgressIterator;
use faster_paths::{
    ch::contractor::ContractedGraph, path::RouteValidationRequest,
    simple_algorithms::ch_bi_dijkstra::ChDijkstra,
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of contracted_graph (output)
    #[arg(short, long)]
    ch_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
}

fn main() {
    let args = Args::parse();

    let reader = BufReader::new(File::open(args.ch_graph).unwrap());
    let contracted_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();
    let dijkstra = ChDijkstra::new(&contracted_graph);

    let reader = BufReader::new(File::open(args.tests_path.as_str()).unwrap());
    let tests: Vec<RouteValidationRequest> = serde_json::from_reader(reader).unwrap();

    let mut times = Vec::new();
    for test in tests.iter().progress() {
        let before = Instant::now();
        let cost = dijkstra.get_cost(&test.request);
        times.push(before.elapsed());

        assert_eq!(cost, test.cost);
    }

    println!("all correct");
    println!(
        "average time was {:?}",
        times.iter().sum::<Duration>() / times.len() as u32
    );
}
