use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::contractor::Contractor,
    graphs::{graph_factory::GraphFactory, path::ShortestPathValidation},
    simple_algorithms::ch_bi_dijkstra::ChDijkstra,
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

    let letters = vec!["ED", "E"];
    for letters in letters {
        let start = Instant::now();
        let contracted_graph = Contractor::get_contracted_graph(&graph, "ED");
        let ch_time = start.elapsed();

        let dijkstra = ChDijkstra::new(&contracted_graph);
        let mut times = Vec::new();
        for test in tests.iter() {
            let before = Instant::now();
            let _ = dijkstra.get_cost(&test.request);
            times.push(before.elapsed());
        }
        let query_time: Duration = (times.iter().sum::<Duration>()) / (times.len() as u32);
        println!(
            "{:<5} ch construction: {:>9} s {:>9} microseconds",
            letters,
            ch_time.as_secs(),
            query_time.as_micros()
        );
    }

    // let writer = BufWriter::new(File::create(args.ch_graph).unwrap());
    // bincode::serialize_into(writer, &contracted_graph).unwrap();
}
