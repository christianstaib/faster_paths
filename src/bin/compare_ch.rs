use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::{
        ch_path_finder::ChPathFinder,
        contractor::serial_contractor::SerialContractor,
        preprocessor::Preprocessor,
        shortcut_replacer::{slow_shortcut_replacer::SlowShortcutReplacer, ShortcutReplacer},
    },
    graphs::{
        graph_factory::GraphFactory,
        path::{PathFinding, ShortestPathValidation},
    },
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
    /// Path of contracted_graph (output)
    #[arg(short, long)]
    ch_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_gr_file(args.graph_path.as_str());
    let reader = BufReader::new(File::open(args.tests_path.as_str()).unwrap());
    let tests: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();

    let letters = vec!["E", "EC", "ED"]; //, "ES", "EC", "EDS", "EDC", "EDCS"];

    for letters in letters {
        let contractor = Box::new(SerialContractor::new(letters));
        let preprocessor = Preprocessor::with_contractor(contractor);

        let start = Instant::now();
        let contracted_graph = preprocessor.get_ch(&graph);
        let ch_time = start.elapsed();

        let shortcut_replacer: Box<dyn ShortcutReplacer> =
            Box::new(SlowShortcutReplacer::new(&contracted_graph.shortcuts));

        let ch = ChPathFinder::new(&contracted_graph.ch_graph, &shortcut_replacer);
        let mut times = Vec::new();
        for test in tests.iter() {
            let before = Instant::now();
            let _ = ch.get_shortest_path_weight(&test.request);
            times.push(before.elapsed());
        }
        let query_time: Duration = (times.iter().sum::<Duration>()) / (times.len() as u32);
        println!(
            "{:<5} ch construction: {:>9} s {:?}",
            letters,
            ch_time.as_secs(),
            query_time
        );
    }

    // let writer = BufWriter::new(File::create(args.ch_graph).unwrap());
    // bincode::serialize_into(writer, &contracted_graph).unwrap();
}
