use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use fast_paths::InputGraph;
use faster_paths::graphs::{graph_factory::GraphFactory, path::ShortestPathValidation};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_gr_file(args.graph_path.as_str());

    let reader = BufReader::new(File::open(args.tests_path.as_str()).unwrap());
    let tests: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();

    let mut input_graph = InputGraph::new();
    for edge in graph.all_edges() {
        input_graph.add_edge(edge.tail as usize, edge.head as usize, edge.weight as usize);
    }

    let start = Instant::now();
    input_graph.freeze();
    let fast_graph = fast_paths::prepare(&input_graph);
    println!("Generating ch took {:?}", start.elapsed());

    let mut times = Vec::new();
    for test in tests.iter() {
        let before = Instant::now();
        let _ = fast_paths::calc_path(
            &fast_graph,
            test.request.source() as usize,
            test.request.target() as usize,
        );
        times.push(before.elapsed());
    }
    let query_time: Duration = (times.iter().sum::<Duration>()) / (times.len() as u32);
    println!("avg time was {:?}", query_time);
}
