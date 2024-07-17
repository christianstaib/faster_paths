use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::graphs::{
    graph_factory::GraphFactory,
    graph_functions::{generate_dijkstra_rank_test_cases, generate_random_pair_test_cases},
};

/// Generates `number_of_tests` many random pair test cases for the graph
/// specified at `graph`. The test cases will be saved at `random_pairs`. For
/// larger `number_of_tests` and complex `graph`s this may take a while.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Graph in `.fmi` of `.gr` format
    #[arg(short, long)]
    graph: PathBuf,
    /// Path where the test cases will be saved
    #[arg(short, long)]
    random_tests: PathBuf,
    /// Path where the test cases will be saved
    #[arg(short, long)]
    rank_tests: PathBuf,
    /// Number of tests to be generated
    #[arg(short, long, default_value = "1000")]
    number_of_tests: u32,
}

fn main() {
    let args = Args::parse();

    println!("Loading Graph");
    let graph = GraphFactory::from_file(&args.graph);

    println!("Generating random pair test cases");
    let start = Instant::now();
    let random_pairs = generate_random_pair_test_cases(&graph, args.number_of_tests);

    let rank_pairs = generate_dijkstra_rank_test_cases(&graph, args.number_of_tests, &random_pairs);
    println!("took {:?}", start.elapsed());

    println!("Writing test cases to file");
    let mut writer = BufWriter::new(File::create(&args.random_tests).unwrap());
    serde_json::to_writer(&mut writer, &random_pairs).unwrap();

    let mut writer = BufWriter::new(File::create(&args.rank_tests).unwrap());
    serde_json::to_writer(&mut writer, &rank_pairs).unwrap();
}
