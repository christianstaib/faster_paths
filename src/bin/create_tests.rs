use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use faster_paths::{
    classical_search::{cache_dijkstra::CacheDijkstra, dijkstra::Dijkstra},
    dijkstra_data::DijkstraData,
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::{
            generate_random_pair_testcases, hitting_set, random_paths, shortests_path_tree,
        },
        path::{ShortestPathRequest, ShortestPathTestCase},
        Graph,
    },
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rand::{thread_rng, Rng};
use rayon::prelude::*;

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
    random_pairs: PathBuf,
    /// TODO
    #[arg(short, long)]
    dijkstra_rank_pairs: PathBuf,
    /// Number of tests to be generated
    #[arg(short, long)]
    number_of_tests: u32,
}

fn main() {
    let args = Args::parse();

    println!("Loading graph");
    let graph = GraphFactory::from_file(&args.graph);

    println!("Loading test cases");
    let reader = BufReader::new(File::open(&args.random_pairs).unwrap());
    let random_pairs: Vec<ShortestPathTestCase> = serde_json::from_reader(reader).unwrap();

    let dijkstra = Dijkstra::new(&graph);

    let number_of_random_pairs = 50_000;
    println!("Generating {} random paths", number_of_random_pairs);
    let paths = random_paths(
        number_of_random_pairs,
        graph.number_of_vertices(),
        &dijkstra,
    );

    println!("generating hitting set");
    let (hitting_setx, _) = hitting_set(&paths, graph.number_of_vertices());

    println!("generating random pair test");

    println!("generating cache");
    let graph: &dyn Graph = &graph;
    let mut dijkstra = CacheDijkstra::new(graph);
    dijkstra.cache = hitting_setx
        .par_iter()
        .progress()
        .map(|&vertex| {
            let data = dijkstra.single_source(vertex);
            let tree = shortests_path_tree(&data);
            let data = data.vertices;
            (vertex, (data, tree))
        })
        .collect();

    let mut times = Vec::new();
    // to beat
    // ny 43.580883ms
    //
    // aegeis average query time with cache 187.731216ms
    // without 453.373613ms
    random_pairs
        .iter()
        .take(2_000)
        .progress()
        .for_each(|test_case| {
            let source = test_case.request.source();
            let target = test_case.request.target();

            let request = ShortestPathRequest::new(source, target).unwrap();

            let start = Instant::now();
            let data = dijkstra.single_source(request.source());
            times.push(start.elapsed());

            let path = data.get_path(target);

            let mut weight = None;
            if let Some(path) = path {
                weight = Some(path.weight);
            }

            assert_eq!(weight, test_case.weight);
        });

    println!(
        "average query time {:?}",
        times.iter().sum::<Duration>() / times.len() as u32
    );

    // let mut writer =
    // BufWriter::new(File::create(&args.random_pairs).unwrap());
    // serde_json::to_writer_pretty(&mut writer, &random_pairs).unwrap();
    // writer.flush().unwrap();
}
