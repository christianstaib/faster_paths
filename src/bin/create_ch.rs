use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::Instant,
};

use clap::Parser;
use faster_paths::{
    ch::{
        ch_dijkstra::ChDijkstra,
        contraction_adaptive_simulated::contract_adaptive_simulated_with_witness,
    },
    graphs::{
        graph_factory::GraphFactory, graph_functions::validate_and_time, path::ShortestPathTestCase,
    },
    shortcut_replacer::slow_shortcut_replacer::SlowShortcutReplacer,
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    infile: PathBuf,
    /// Testcase file
    #[arg(short, long)]
    tests: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    outfile: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("loading test cases");
    let reader = BufReader::new(File::open(&args.tests).unwrap());
    let test_cases: Vec<ShortestPathTestCase> = serde_json::from_reader(reader).unwrap();

    println!("loading graph");
    let graph = GraphFactory::from_file(&args.infile);

    let start = Instant::now();
    let ch_and_shortctus = contract_adaptive_simulated_with_witness(&graph);
    println!("it took {:?} to contract the graph", start.elapsed());

    println!("writing contracted graph to file");
    let writer = BufWriter::new(File::create(args.outfile).unwrap());
    bincode::serialize_into(writer, &ch_and_shortctus).unwrap();

    let (contracted_graph, shortcuts) = ch_and_shortctus;

    // setting up path finder
    let ch_dijkstra = ChDijkstra::new(&contracted_graph);
    let path_finder = SlowShortcutReplacer::new(&shortcuts, &ch_dijkstra);

    let average_query_time = validate_and_time(&test_cases, &path_finder, &graph);

    println!(
        "All {} tests passed. Average query time was {:?}",
        test_cases.len(),
        average_query_time
    );
}
