use std::{fs::File, io::BufReader, path::PathBuf, time::Duration};

use clap::Parser;
use faster_paths::graphs::{
    graph_factory::GraphFactory,
    graph_functions::validate_and_time,
    path::{read_pathfinder, ShortestPathTestCase},
};

/// Generates `number_of_tests` many random pair test cases for the graph
/// specified at `graph`. The test cases will be saved at `random_pairs`. For
/// larger `number_of_tests` and complex `graph`s this may take a while.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Graph, CH, or HL
    #[arg(short, long)]
    pathfinder: PathBuf,
    /// Graph in `.fmi` of `.gr` format
    #[arg(short, long)]
    graph: PathBuf,
    /// Path where the test cases will be saved
    #[arg(short, long)]
    test_cases: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Reading test cases");
    let mut reader = BufReader::new(File::open(&args.test_cases).unwrap());
    let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(&mut reader).unwrap();

    println!("Reading graph");
    let graph = GraphFactory::from_file(&args.graph);

    println!("Reading pathfinder");
    let path_finder = read_pathfinder(&args.pathfinder).unwrap();

    println!("Testing & validating");
    let results = validate_and_time(&test_cases, &*path_finder, &graph);
    let average: f64 = results
        .iter()
        .map(|result| result.timing_in_seconds)
        .sum::<f64>()
        / results.len() as f64;
    let average = Duration::from_secs_f64(average);

    println!(
        "All correct. Took {:?} per query averaged over {} queries",
        average,
        test_cases.len()
    );
}
