use std::{
    fs::File,
    io::BufReader,
    time::{self, Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::preprocessor::ContractedGraph,
    graphs::{
        fast_graph::FastGraph,
        graph_factory::GraphFactory,
        path::{PathFinding, ShortestPathValidation},
    },
    hl::hub_graph::HubGraph,
    simple_algorithms::{
        bi_dijkstra::BiDijkstra,
        ch_bi_dijkstra::ChDijkstra,
        dijkstra::Dijkstra,
        fast_dijkstra::{self, FastDijkstra},
        slow_dijkstra::SlowDijkstra,
    },
};
use indicatif::ProgressIterator;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    ch_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    hl_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
}

fn main() {
    let args = Args::parse();

    let slow_graph = GraphFactory::from_gr_file(args.graph_path.as_str());
    let graph = FastGraph::from_graph(&slow_graph);

    let slow_dijkstra = SlowDijkstra::new(&slow_graph);

    let dijkstra = Dijkstra::new(&graph);
    let fast_dijkstra = FastDijkstra::new(&graph);

    // let bi_dijkstra = BiDijkstra::new(&graph);

    // let reader = BufReader::new(File::open(args.ch_path).unwrap());
    // let ch_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();
    // let ch = ChDijkstra::new(&ch_graph);

    // let reader = BufReader::new(File::open(args.hl_path).unwrap());
    // let hl: HubGraph = bincode::deserialize_from(reader).unwrap();

    let reader = BufReader::new(File::open(args.tests_path).unwrap());
    let validations: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();

    let mut path_finder: Vec<(&str, Box<dyn PathFinding>, Vec<Duration>)> = Vec::new();
    // path_finder.push(("dijkstra", Box::new(dijkstra), Vec::new()));
    // path_finder.push(("slow dijkstra", Box::new(slow_dijkstra), Vec::new()));
    path_finder.push(("fast dijkstra", Box::new(fast_dijkstra), Vec::new()));
    // path_finder.push(("bi dijkstra", Box::new(bi_dijkstra), Vec::new()));
    // path_finder.push(("ch", Box::new(ch), Vec::new()));
    // path_finder.push(("hl", Box::new(hl), Vec::new()));

    for validation in validations.iter().take(100).progress() {
        for (name, path_finder, times) in path_finder.iter_mut() {
            let start = Instant::now();
            let path = path_finder.get_shortest_path(&validation.request);
            times.push(start.elapsed());

            if let Err(err) = slow_graph.validate_path(&validation, &path) {
                panic!("{} wrong: {}", name, err);
            }
        }
    }

    for (name, _, times) in path_finder.iter() {
        let average = times.iter().sum::<Duration>() / times.len() as u32;
        println!("{:<15} {:?}", name, average);
    }
}
