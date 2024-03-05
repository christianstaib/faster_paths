use std::{fs::File, io::BufRead, io::BufReader};

use clap::Parser;
use faster_paths::{
    graphs::fast_graph::FastGraph,
    graphs::graph_factory::GraphFactory,
    graphs::path::{Routing, ShortestPathRequest},
    simple_algorithms::dijkstra::Dijkstra,
};
use indicatif::ProgressIterator;

/// Tests the dijkstra implementation against a known distances.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of graph (.fmi file).
    #[arg(short, long)]
    graph_path: String,
    /// Path of file with source, target pairs (.que file).
    #[arg(short, long)]
    queue_path: String,
    /// Path of file with known distances. (.fmi file).
    #[arg(short, long)]
    sol_path: String,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_fmi_file(args.graph_path.as_str());

    let fast_graph = FastGraph::from_graph(&graph);
    let dijkstra = Dijkstra::new(&fast_graph);

    let queue: Vec<_> = BufReader::new(File::open(args.queue_path).unwrap())
        .lines()
        .map(|s| s.ok())
        .filter_map(|s| s)
        .map(|s| {
            s.split_whitespace()
                .map(|num| num.parse::<u32>().unwrap())
                .collect::<Vec<_>>()
        })
        .collect();

    let sol: Vec<_> = BufReader::new(File::open(args.sol_path).unwrap())
        .lines()
        .map(|s| s.ok())
        .filter_map(|s| s)
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    for (source_target, true_cost) in queue.iter().zip(sol.iter()).progress() {
        let request = ShortestPathRequest::new(source_target[0], source_target[1]).unwrap();

        // test dijkstra
        let response = dijkstra.get_shortest_path(&request);
        let mut cost: i32 = -1;
        if let Some(route) = response {
            cost = route.weight as i32;
        }
        assert_eq!(true_cost, &cost, "dijkstra wrong");
    }

    println!("dijkstra correct");
}
