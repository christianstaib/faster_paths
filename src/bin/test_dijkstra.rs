use std::{fs::File, io::BufRead, io::BufReader};

use clap::Parser;
use faster_paths::{
    ch::{
        ch_path_finder::ChPathFinder, preprocessor::Preprocessor,
        shortcut_replacer::slow_shortcut_replacer::SlowShortcutReplacer,
    },
    graphs::{
        fast_graph::FastGraph,
        graph_factory::GraphFactory,
        path::{PathFinding, ShortestPathRequest},
    },
    hl::{hub_graph_factory::HubGraphFactory, hub_graph_path_finder::HubGraphPathFinder},
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
    let dijkstra = Dijkstra::new(fast_graph);

    let preprocessor = Preprocessor::new();
    let contracted_graph = preprocessor.get_ch(&graph);
    let shortcut_replacer: Box<_> =
        Box::new(SlowShortcutReplacer::new(&contracted_graph.shortcuts));

    let ch = ChPathFinder::new(contracted_graph.ch_graph.clone(), shortcut_replacer.clone());
    let hl = HubGraphFactory::new(&contracted_graph);
    let hl = hl.get_hl();
    let hl = HubGraphPathFinder::new(hl, shortcut_replacer);

    let queue: Vec<_> = BufReader::new(File::open(args.queue_path).unwrap())
        .lines()
        .map_while(Result::ok)
        .map(|s| {
            s.split_whitespace()
                .map(|num| num.parse::<u32>().unwrap())
                .collect::<Vec<_>>()
        })
        .collect();

    let sol: Vec<_> = BufReader::new(File::open(args.sol_path).unwrap())
        .lines()
        .map_while(Result::ok)
        .map(|s| s.parse::<i32>().unwrap())
        .collect();

    for (source_target, true_cost) in queue.iter().zip(sol.iter()).progress() {
        let request = ShortestPathRequest::new(source_target[0], source_target[1]).unwrap();

        // test dijkstra
        let path = dijkstra.get_shortest_path(&request);
        let mut cost: i32 = -1;
        if let Some(route) = path {
            cost = route.weight as i32;
        }
        assert_eq!(
            true_cost, &cost,
            "Dijkstra wrong. Weight of path should be {} but is {}",
            true_cost, cost
        );

        let ch_path = ch.get_shortest_path(&request);
        if let Some(ch_path) = ch_path {
            assert_eq!(cost, ch_path.weight as i32);
        } else {
            assert_eq!(true_cost, &-1);
        }

        let hl_path = hl.get_shortest_path(&request);
        if let Some(hl_path) = hl_path {
            assert_eq!(cost, hl_path.weight as i32);
        } else {
            assert_eq!(true_cost, &-1);
        }
    }

    println!("dijkstra correct");
}
