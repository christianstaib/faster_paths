use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::preprocessor::ContractedGraph,
    graphs::path::{PathFinding, ShortestPathValidation},
    simple_algorithms::ch_bi_dijkstra::ChDijkstra,
};
use indicatif::ProgressIterator;

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
    let tests: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();

    let mut times = Vec::new();
    for test in tests.iter().progress() {
        let before = Instant::now();
        let path = dijkstra.get_shortest_path(&test.request);
        let mut cost = None;
        if let Some(path) = path {
            cost = Some(path.weight);
        }
        times.push(before.elapsed());

        assert_eq!(cost, test.weight);
    }

    println!("all correct");
    println!(
        "average time was {:?}",
        times.iter().sum::<Duration>() / times.len() as u32
    );
}
