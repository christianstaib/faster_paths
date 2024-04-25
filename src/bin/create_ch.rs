use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    ch::{
        ch_dijkstra::ChDijkstra,
        preprocessor::ch_with_witness,
    },
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::all_edges,
        path::{PathFinding, ShortestPathTestCase},
        reversible_hash_graph::ReversibleHashGraph,
        Graph,
    },
};
use indicatif::ProgressIterator;

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    infile: PathBuf,
    /// Path of .fmi file
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

    println!(
        "average vertex degree of base graph is {}",
        graph.number_of_edges() as f32 / graph.number_of_vertices() as f32
    );

    println!("switching graph represenation");
    let working_graph = ReversibleHashGraph::from_edges(&all_edges(&graph));

    println!("starting ch");
    let boxed_graph = Box::new(working_graph);

    let contracted_graph = ch_with_witness(boxed_graph);

    // let mut preprocessor = AllInPrerocessor {};
    // let contracted_graph = preprocessor.get_ch(boxed_graph);

    println!("writing ch to file");
    let writer = BufWriter::new(File::create(args.outfile).unwrap());
    bincode::serialize_into(writer, &contracted_graph).unwrap();

    println!(
        "average vertex degree of upward_graph is {}",
        contracted_graph.upward_graph.number_of_edges() as f32
            / contracted_graph.upward_graph.number_of_vertices() as f32
    );

    let ch_dijkstra = ChDijkstra::new(&contracted_graph);
    let path_finder: Box<dyn PathFinding> = Box::new(ch_dijkstra);

    let mut times = Vec::new();
    for test_case in test_cases.iter().progress() {
        let start = Instant::now();
        let _path = path_finder.shortest_path_weight(&test_case.request);
        times.push(start.elapsed());

        if _path != test_case.weight {
            println!("err soll {:?}, ist {:?}", test_case.weight, _path);
        }
    }

    println!("all {} tests passed", test_cases.len());

    let average = times.iter().sum::<Duration>() / times.len() as u32;
    println!(
        "the average query time over {} queries was {:?}",
        test_cases.len(),
        average
    );
}
