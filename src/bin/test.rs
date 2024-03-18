use std::{
    fs::File,
    io::BufReader,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::{
        ch_path_finder::ChPathFinder,
        shortcut_replacer::{
            fast_shortcut_replacer::FastShortcutReplacer,
            slow_shortcut_replacer::SlowShortcutReplacer, ShortcutReplacer,
        },
        ContractedGraphInformation,
    },
    graphs::{
        fast_graph::FastGraph,
        graph_factory::GraphFactory,
        path::{PathFinding, ShortestPathValidation},
    },
    hl::{hub_graph::HubGraph, hub_graph_path_finder::HubGraphPathFinder},
    simple_algorithms::{
        dijkstra::Dijkstra, fast_dijkstra::FastDijkstra, slow_dijkstra::SlowDijkstra,
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
    let fast_graph = FastGraph::from_graph(&slow_graph);

    let mut path_finder: Vec<(&str, Box<dyn PathFinding>, Vec<Duration>)> = Vec::new();

    let dijkstra = Dijkstra::new(fast_graph.clone());
    path_finder.push(("dijkstra", Box::new(dijkstra), Vec::new()));

    // ch
    let ch_reader = BufReader::new(File::open(args.ch_path).unwrap());
    let ch: ContractedGraphInformation = bincode::deserialize_from(ch_reader).unwrap();

    let shortcut_replacer: Box<dyn ShortcutReplacer> =
        Box::new(FastShortcutReplacer::new(&ch.shortcuts));
    let ch_path_finder = ChPathFinder::new(ch.ch_graph.clone(), shortcut_replacer);
    path_finder.push(("ch", Box::new(ch_path_finder), Vec::new()));

    // hl
    let shortcut_replacer: Box<dyn ShortcutReplacer> =
        Box::new(FastShortcutReplacer::new(&ch.shortcuts));
    let hl_reader = BufReader::new(File::open(args.hl_path).unwrap());
    let hl: HubGraph = bincode::deserialize_from(hl_reader).unwrap();
    let hl_path_finder = HubGraphPathFinder::new(hl, shortcut_replacer);
    path_finder.push(("hl", Box::new(hl_path_finder), Vec::new()));

    let reader = BufReader::new(File::open(args.tests_path).unwrap());
    let validations: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();

    for (name, path_finder, times) in path_finder.iter_mut() {
        println!("testing {}", name);
        for validation in validations.iter().progress() {
            let start = Instant::now();
            let path = path_finder.get_shortest_path(&validation.request);
            times.push(start.elapsed());

            // if let Err(err) = slow_graph.validate_path(&validation, &path) {
            //     panic!("{} wrong: {}", name, err);
            // }
        }
    }

    for (name, _, times) in path_finder.iter() {
        let average = times.iter().sum::<Duration>() / times.len() as u32;
        println!("{:<15} {:?}", name, average);
    }
}
