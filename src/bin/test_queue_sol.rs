use std::{fs::File, io::BufRead, io::BufReader};

use clap::Parser;
use indicatif::ProgressIterator;
use faster_paths::{
    ch::contractor::{ContractedGraph, Contractor},
    fast_graph::FastGraph,
    graph::Graph,
    hl::hub_graph::HubGraph,
    naive_graph::NaiveGraph,
    path::{PathRequest, Routing},
    simple_algorithms::{bi_dijkstra::BiDijkstra, ch_bi_dijkstra::ChDijkstra, dijkstra::Dijkstra},
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    queue_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    sol_path: String,
}

fn main() {
    let args = Args::parse();

    let naive_graph = NaiveGraph::from_fmi_file(args.graph_path.as_str());
    let graph = Graph::from_edges(&naive_graph.edges);

    let ch_graph = Contractor::get_contracted_graph(&graph);
    let ch_bi_dijkstra = ChDijkstra::new(&ch_graph);

    let fast_graph = FastGraph::from_graph(&graph);

    let dijkstra = Dijkstra::new(&fast_graph);

    let bi_dijkstra = BiDijkstra::new(&fast_graph);

    let hl_graph = ch_bi_dijkstra.get_hl();

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
        let request = PathRequest {
            source: source_target[0],
            target: source_target[1],
        };

        // test dijkstra
        let response = dijkstra.get_path(&request);
        let mut cost: i32 = -1;
        if let Some(route) = response {
            cost = route.weight as i32;
        }
        assert_eq!(true_cost, &cost, "dijkstra wrong");

        // test bi dijkstra
        let response = bi_dijkstra.get_path(&request);
        let mut cost: i32 = -1;
        if let Some(route) = response {
            cost = route.weight as i32;
        }
        assert_eq!(true_cost, &cost, "bi dijkstra wrong");

        // test ch dijkstra
        let response = ch_bi_dijkstra.get_route(&request);
        let mut cost: i32 = -1;
        if let Some(route) = response {
            cost = route.weight as i32;
        }
        assert_eq!(true_cost, &cost, "ch dijkstra wrong");

        // test hl
        let response = hl_graph.get_weight(&request);
        let mut cost: i32 = -1;
        if let Some(this_cost) = response {
            cost = this_cost as i32;
        }
        assert_eq!(true_cost, &cost, "hl wrong");
    }
}
