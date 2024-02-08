use std::{fs::File, io::BufReader};

use clap::Parser;
use osm_test::{
    ch::contractor::ContractedGraph,
    fast_graph::FastGraph,
    graph::Graph,
    hl::hub_graph::HubGraph,
    naive_graph::NaiveGraph,
    path::{RouteValidationRequest, Routing},
    simple_algorithms::{bi_dijkstra::BiDijkstra, ch_bi_dijkstra::ChDijkstra, dijkstra::Dijkstra},
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_ch_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    fmi_hl_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
}

fn main() {
    let args = Args::parse();

    let graph = NaiveGraph::from_fmi_file(args.fmi_path.as_str());
    let graph = Graph::from_edges(&graph.edges);
    let graph = FastGraph::from_graph(&graph);

    let dijkstra = Dijkstra::new(&graph);

    let bi_dijkstra = BiDijkstra::new(&graph);

    let reader = BufReader::new(File::open(args.fmi_ch_path).unwrap());
    let ch_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();
    let ch_bi_dijkstra = ChDijkstra::new(&ch_graph);

    let reader = BufReader::new(File::open(args.fmi_hl_path).unwrap());
    let hl_graph: HubGraph = bincode::deserialize_from(reader).unwrap();

    let reader = BufReader::new(File::open(args.tests_path).unwrap());
    let requests: Vec<RouteValidationRequest> = serde_json::from_reader(reader).unwrap();

    for request in requests.iter() {
        let true_cost = request.cost;
        let request = request.request.clone();

        // test dijkstra
        let response = dijkstra.get_route(&request);
        let mut cost = None;
        if let Some(route) = response.route {
            cost = Some(route.weight);
        }
        assert_eq!(true_cost, cost, "dijkstra wrong");

        // test bi dijkstra
        let response = bi_dijkstra.get_route(&request);
        let mut cost = None;
        if let Some(route) = response.route {
            cost = Some(route.weight);
        }
        assert_eq!(true_cost, cost, "bi dijkstra wrong");

        // test ch dijkstra
        let response = ch_bi_dijkstra.get_route(&request);
        let mut cost = None;
        if let Some(route) = response {
            cost = Some(route.weight);
        }
        assert_eq!(true_cost, cost, "ch dijkstra wrong");

        // test hl
        let response = hl_graph.get_weight(&request);
        let cost = response;
        assert_eq!(true_cost, cost, "bi dijkstra wrong");
    }
}
