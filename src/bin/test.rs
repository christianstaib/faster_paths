use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use faster_paths::{
    ch::directed_contracted_graph::DirectedContractedGraph,
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::validate_and_time,
        path::{PathFinding, ShortestPathTestCase},
    },
    hl::directed_hub_graph::DirectedHubGraph,
};

/// Generates `number_of_tests` many random pair test cases for the graph
/// specified at `graph`. The test cases will be saved at `random_pairs`. For
/// larger `number_of_tests` and complex `graph`s this may take a while.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Graph in `.fmi` of `.gr` format
    #[arg(short, long)]
    pathfinder: PathBuf,
    /// Graph in `.fmi` of `.gr` format
    #[arg(short, long)]
    graph: PathBuf,
    /// Path where the test cases will be saved
    #[arg(short, long)]
    random_pairs: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Reading test cases");
    let mut reader = BufReader::new(File::open(&args.random_pairs).unwrap());
    let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(&mut reader).unwrap();

    println!("Reading graph");
    let graph = GraphFactory::from_file(&args.graph);

    println!("Reading pathfinder");
    let path_finder: Box<dyn PathFinding>;
    let reader = BufReader::new(File::open(&args.pathfinder).unwrap());

    let pathfinder_string = args.pathfinder.to_str().unwrap();
    if pathfinder_string.ends_with(".di.ch.bincode") {
        let contracted_graph: DirectedContractedGraph = bincode::deserialize_from(reader).unwrap();
        path_finder = Box::new(contracted_graph);
    } else if pathfinder_string.ends_with(".di.hl.bincode") {
        let hub_graph: DirectedHubGraph = bincode::deserialize_from(reader).unwrap();
        path_finder = Box::new(hub_graph);
    } else {
        panic!("cant read file \"{}\"", args.pathfinder.to_str().unwrap());
    }

    println!("Generating random pair test cases");
    let average_time = validate_and_time(&test_cases, &*path_finder, &graph);
    println!(
        "took {:?} per query averaged over {} queries",
        average_time,
        test_cases.len()
    );
}
