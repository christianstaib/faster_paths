use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use faster_paths::graphs::{
    graph_factory::GraphFactory,
    graph_functions::{hitting_set, random_paths, test_cases},
    Graph,
};
use itertools::Itertools;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    random_pairs: PathBuf,
    /// Path of .fmi file
    #[arg(short, long)]
    dijkstra_rank_pairs: PathBuf,
    /// Number of tests to be run
    #[arg(short, long)]
    number_of_tests: u32,
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_file(&args.graph);

    let paths = random_paths(args.number_of_tests, &graph);

    let hitting_set = hitting_set(&paths, graph.number_of_vertices());
    println!("{:?}", hitting_set.iter().take(100).collect_vec());

    println!(
        "hitted {} out of {} vertices",
        hitting_set.len(),
        graph.number_of_vertices()
    );

    println!("generating random pair test");
    let start = Instant::now();
    let random_pairs = test_cases(args.number_of_tests, &graph);
    println!("took {:?}", start.elapsed());

    let mut writer = BufWriter::new(File::create(&args.random_pairs).unwrap());
    serde_json::to_writer_pretty(&mut writer, &random_pairs).unwrap();
    writer.flush().unwrap();
}
