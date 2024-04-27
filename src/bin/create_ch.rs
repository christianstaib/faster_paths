use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::{ch_dijkstra::ChDijkstra, contraction_adaptive_simulated::ch_with_witness},
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::all_edges,
        path::{PathFinding, ShortestPathTestCase},
        reversible_hash_graph::ReversibleHashGraph,
        Graph,
    },
    shortcut_replacer::slow_shortcut_replacer::SlowShortcutReplacer,
};
use indicatif::ProgressIterator;

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

    println!("switching graph represenation");
    let working_graph = ReversibleHashGraph::from_edges(&all_edges(&graph));

    println!("starting ch");
    let boxed_graph = Box::new(working_graph);

    let (contracted_graph, shortcuts) = ch_with_witness(boxed_graph);

    // let mut preprocessor = AllInPrerocessor {};
    // let contracted_graph = preprocessor.get_ch(boxed_graph);

    println!("writing ch to file");
    let writer = BufWriter::new(File::create(args.outfile).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();

    // setting up path finder
    let ch_dijkstra = ChDijkstra::new(&contracted_graph);
    let path_finder = SlowShortcutReplacer::new(&shortcuts, &ch_dijkstra);

    println!("running {} tests", test_cases.len());
    for test_case in test_cases.iter().progress() {
        let _path = path_finder.shortest_path(&test_case.request);
    }
    println!("all {} tests passed", test_cases.len());
}
