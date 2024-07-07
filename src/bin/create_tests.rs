use std::{env::temp_dir, fs::File, io::BufWriter, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::graphs::{
    graph_factory::GraphFactory, graph_functions::generate_random_pair_test_cases,
    path::ShortestPathTestCaseC,
};
use itertools::Itertools;

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
    test_cases: PathBuf,
    /// Number of tests to be generated
    #[arg(short, long)]
    number_of_tests: u32,
}

fn main() {
    let args = Args::parse();

    println!("Loading Graph");
    let graph = GraphFactory::from_file(&args.graph);

    println!("Generating random pair test cases");
    let start = Instant::now();
    let random_pairs = generate_random_pair_test_cases(&graph, args.number_of_tests);
    println!("took {:?}", start.elapsed());

    println!("Writing test cases to file");
    let mut writer = BufWriter::new(File::create(&args.test_cases).unwrap());
    serde_json::to_writer(&mut writer, &random_pairs).unwrap();
}
