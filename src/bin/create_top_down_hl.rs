use std::{
    cmp::Reverse,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    time::{Duration, Instant},
};

use ahash::{HashMap, HashMapExt};
use clap::Parser;
use faster_paths::{
    classical_search::dijkstra::Dijkstra,
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::{hitting_set, random_paths, validate_and_time, validate_path},
        path::{PathFinding, ShortestPathTestCase},
        reversible_vec_graph::ReversibleVecGraph,
        Graph,
    },
    hl::{
        hl_path_finding::{shortest_path, HLPathFinder},
        top_down_hl::{generate_forward_label, generate_hub_graph, generate_reverse_label},
    },
    shortcut_replacer::slow_shortcut_replacer::{replace_shortcuts_slow, SlowShortcutReplacer},
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};

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

    let number_of_random_pairs = 5_000;
    println!("Generating {} random paths", number_of_random_pairs);
    let dijkstra = Dijkstra::new(&graph);
    let paths = random_paths(number_of_random_pairs, &graph, &dijkstra);

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

    let n = 1_000;
    println!("Predicting average label size over {} vertices", n);
    let average_label_size = predict_average_label_size(&test_cases, n, &graph, &order);
    println!("Average label size is {} ", average_label_size);

    println!("Generating hub graph");
    let hub_graph_and_shortcuts = generate_hub_graph(&graph, &order);

    println!("Saving hub graph as bincode");
    let writer = BufWriter::new(File::create("hl_test.bincode").unwrap());
    bincode::serialize_into(writer, &hub_graph_and_shortcuts).unwrap();

    println!("Saving hub graph as json");
    let writer = BufWriter::new(File::create("hl_test.json").unwrap());
    serde_json::to_writer(writer, &hub_graph_and_shortcuts).unwrap();

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

fn predict_average_label_size(
    test_cases: &Vec<ShortestPathTestCase>,
    number_of_labels: u32,
    graph: &dyn Graph,
    order: &Vec<u32>,
) -> f64 {
    let labels: Vec<_> = test_cases
        .par_iter()
        .take(number_of_labels as usize)
        .progress()
        .map(|test_case| {
            let mut shortcuts = HashMap::new();

            let (forward_label, forward_shortcuts) =
                generate_forward_label(test_case.request.source(), graph, order);
            let (reverse_label, reverse_shortcuts) =
                generate_reverse_label(test_case.request.target(), graph, order);

            shortcuts.extend(forward_shortcuts.iter().cloned());
            shortcuts.extend(
                reverse_shortcuts
                    .into_iter()
                    .map(|(x, y)| (x.reversed(), y)),
            );

            let mut path = shortest_path(&forward_label, &reverse_label);

            // if there exits a path, replace the shortcuts on it
            if let Some(ref mut path) = path {
                replace_shortcuts_slow(&mut path.vertices, &shortcuts);
            }

            if let Err(err) = validate_path(graph, test_case, &path) {
                panic!("{}", err);
            }

            vec![forward_label, reverse_label]
        })
        .flatten()
        .collect();

    let average_label_size =
        labels.iter().map(|l| l.entries.len()).sum::<usize>() as f64 / labels.len() as f64;
    average_label_size
}
