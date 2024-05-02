use std::{
    cmp::Reverse,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::{Duration, Instant},
};

use ahash::{HashSet, HashSetExt};
use clap::Parser;
use faster_paths::{
    classical_search::dijkstra::Dijkstra,
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::{hitting_set, random_paths, validate_and_time},
        path::ShortestPathTestCase,
        reversible_vec_graph::ReversibleVecGraph,
        Graph,
    },
    hl::{
        hl_path_finding::HLPathFinder,
        top_down_hl::{generate_hub_graph, predict_average_label_size},
    },
    shortcut_replacer::slow_shortcut_replacer::SlowShortcutReplacer,
};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// Creates a hub graph top down.
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

    let number_of_random_pairs = 5_000;
    let order = generate_hiting_set_order(number_of_random_pairs, &graph);

    let number_of_vertices_with_labels = 1_000;
    println!(
        "Predicting average label size over {} vertices",
        number_of_vertices_with_labels
    );
    let start = Instant::now();
    let average_label_size =
        predict_average_label_size(&test_cases, number_of_vertices_with_labels, &graph, &order);
    println!("Average label size is {} ", average_label_size);
    println!(
        "Generating all labels will take aorund {:?}",
        start.elapsed() / number_of_vertices_with_labels as u32 * graph.number_of_vertices()
    );

    println!("Generating hub graph");
    let start = Instant::now();
    let hub_graph_and_shortcuts = generate_hub_graph(&graph, &order);
    println!("Generating all labels took {:?}", start.elapsed());

    println!("Saving hub graph as bincode");
    let writer = BufWriter::new(File::create(&args.outfile).unwrap());
    bincode::serialize_into(writer, &hub_graph_and_shortcuts).unwrap();

    // TODO this throws an error as the shortcut hasmap use non string keys.
    // println!("Saving hub graph as json");
    // let writer = BufWriter::new(File::create("hl_test.json").unwrap());
    // serde_json::to_writer(writer, &hub_graph_and_shortcuts).unwrap();

    println!("Testing pathfinding with {} test cases", test_cases.len());
    let (hub_graph, shortcuts) = hub_graph_and_shortcuts;
    let path_finder = HLPathFinder::new(&hub_graph);
    let path_finder = SlowShortcutReplacer::new(&shortcuts, &path_finder);

    let times = validate_and_time(&test_cases, &path_finder, &graph);
    println!(
        "All tests passed. Average query time over {} test cases was {:?}.",
        test_cases.len(),
        times.iter().sum::<Duration>() / times.len() as u32
    );
}

fn generate_hiting_set_order(number_of_random_pairs: u32, graph: &dyn Graph) -> Vec<u32> {
    println!("Generating {} random paths", number_of_random_pairs);
    let dijkstra = Dijkstra::new(graph);
    let paths = random_paths(number_of_random_pairs, graph, &dijkstra);

    println!("generating hitting set");
    let (mut hitting_setx, num_hits) = hitting_set(&paths, graph.number_of_vertices());

    println!("generating vertex order");
    let mut not_hitting_set = (0..graph.number_of_vertices())
        .into_iter()
        .filter(|vertex| !hitting_setx.contains(&vertex))
        .collect_vec();

    // shuffle to break neighboring ties
    not_hitting_set.shuffle(&mut thread_rng());
    not_hitting_set.sort_unstable_by_key(|&vertex| Reverse(num_hits[vertex as usize]));

    hitting_setx.extend(not_hitting_set);
    hitting_setx.reverse();

    let order: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .map(|vertex| hitting_setx.iter().position(|&x| x == vertex).unwrap() as u32)
        .collect();
    order
}
