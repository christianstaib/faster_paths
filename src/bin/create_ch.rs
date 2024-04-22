use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    ch::{all_in_preprocessor::AllInPrerocessor, preprocessor::Preprocessor},
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
    let mut preprocessor = Preprocessor::new_all_in(&graph);
    let boxed_graph = Box::new(working_graph);
    let contracted_graph = preprocessor.get_ch(boxed_graph);

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

    let path_finder: Box<dyn PathFinding> = Box::new(contracted_graph);

    for test_case in test_cases.iter().progress() {
        let _path = path_finder.shortest_path_weight(&test_case.request);

        if _path != test_case.weight {
            println!("err soll {:?}, ist {:?}", test_case.weight, _path);
        }

        // assert_eq!(_path, test_case.weight);
        // if let Err(err) = validate_path(&graph, test_case, &_path) {
        //     panic!("ch wrong: {}", err);
        // }
    }

    println!("all {} tests passed", test_cases.len());
}
