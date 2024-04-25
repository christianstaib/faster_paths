use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::{
        ch_dijkstra::{ChDijkstra},
        contracted_graph::ContractedGraph,
    },
    graphs::path::{PathFinding, ShortestPathTestCase},
};
use indicatif::ProgressIterator;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_ch: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    random_pairs: PathBuf,
}

fn main() {
    let args = Args::parse();

    let graph = ContractedGraph::from_fmi_file(&args.fmi_ch);
    let ch_dijkstra = ChDijkstra::new(&graph);
    let path_finder: Box<dyn PathFinding> = Box::new(ch_dijkstra);

    let reader = BufReader::new(File::open(&args.random_pairs).unwrap());
    let random_pairs: Vec<ShortestPathTestCase> = serde_json::from_reader(reader).unwrap();

    let mut times = Vec::new();
    for validation in random_pairs.iter().progress() {
        let start = Instant::now();
        let _path = path_finder.shortest_path_weight(&validation.request);
        times.push(start.elapsed());

        // assert_eq!(validation.weight, _path);
    }

    let average = times.iter().sum::<Duration>() / times.len() as u32;
    println!(
        "the average query time over {} queries was {:?}",
        random_pairs.len(),
        average
    );
}
