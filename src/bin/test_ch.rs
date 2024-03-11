use core::panic;
use std::{
    fmt::format,
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::preprocessor::ContractedGraph,
    graphs::{
        graph_factory::GraphFactory,
        path::{PathFinding, ShortestPathValidation},
    },
    simple_algorithms::ch_bi_dijkstra::ChDijkstra,
};
use indicatif::ProgressIterator;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    graph: String,
    /// Path of contracted_graph (output)
    #[arg(short, long)]
    ch_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_gr_file(args.graph.as_str());

    let reader = BufReader::new(File::open(args.ch_graph).unwrap());
    let contracted_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();
    let dijkstra = ChDijkstra::new(&contracted_graph);

    let reader = BufReader::new(File::open(args.tests_path.as_str()).unwrap());
    let tests: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();

    let mut times = Vec::new();
    for test in tests.iter().progress() {
        let before = Instant::now();
        let path = dijkstra.get_shortest_path(&test.request);
        times.push(before.elapsed());

        if let Err(err) = graph.validate_path(&test, &path) {
            panic!("{}", err);
        }
    }

    println!("all correct");
    println!(
        "average time was {:?}",
        times.iter().sum::<Duration>() / times.len() as u32
    );
}
