use core::panic;
use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::{
        shortcut_replacer::{self, slow_shortcut_replacer::SlowShortcutReplacer, ShortcutReplacer},
        ContractedGraphInformation,
    },
    graphs::{
        fast_graph::FastGraph,
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
    let contracted_graph: ContractedGraphInformation = bincode::deserialize_from(reader).unwrap();

    let ch_graph = FastGraph::from_graph(&contracted_graph.graph);
    let shortcut_replacer: Box<dyn ShortcutReplacer> =
        Box::new(SlowShortcutReplacer::new(&contracted_graph.shortcuts));
    let dijkstra = ChDijkstra::new(&ch_graph, &shortcut_replacer);

    let reader = BufReader::new(File::open(args.tests_path.as_str()).unwrap());
    let tests: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();

    let mut times = Vec::new();
    for test in tests.iter().take(1000).progress() {
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
