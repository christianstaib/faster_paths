use std::{
    cmp::Reverse,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use ahash::{HashMap, HashMapExt};
use clap::Parser;
use faster_paths::{
    classical_search::dijkstra::Dijkstra,
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::{hitting_set, random_paths, validate_path},
        path::{PathFinding, ShortestPathTestCase},
        Graph,
    },
    hl::{
        hl_path_finding::{shortest_path, HLPathFinder},
        top_down_hl::{generate_hub_graph, get_in_label, get_out_label},
    },
    shortcut_replacer::slow_shortcut_replacer::{replace_shortcuts_slow, SlowShortcutReplacer},
};
use indicatif::ParallelProgressIterator;
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

    let dijkstra = Dijkstra::new(&graph);

    println!("generating random paths");
    let paths = random_paths(5_000, &graph, &dijkstra);

    println!("generating hitting set");
    let (mut hitting_setx, num_hits) = hitting_set(&paths, graph.number_of_vertices());

    println!("generating vertex order");
    let mut not_hitting_set = (0..graph.number_of_vertices())
        .into_iter()
        .filter(|vertex| !hitting_setx.contains(&vertex))
        .collect_vec();
    not_hitting_set.shuffle(&mut thread_rng());
    not_hitting_set.sort_unstable_by_key(|&vertex| Reverse(num_hits[vertex as usize]));

    hitting_setx.extend(not_hitting_set);
    hitting_setx.reverse();

    // let mut order = (0..graph.number_of_vertices()).collect_vec();
    // order.shuffle(&mut rand::thread_rng());
    let order: Vec<_> = (0..graph.number_of_vertices())
        .into_par_iter()
        .map(|vertex| hitting_setx.iter().position(|&x| x == vertex).unwrap() as u32)
        .collect();

    println!("testing logic");
    let average_label_size = predict_average_label_size(&test_cases, &graph, &order);

    println!("average label size is {} ", average_label_size);

    println!("generating hl");
    let (hub_graph, _shortcuts) = generate_hub_graph(&graph, &order);
    let path_finder = HLPathFinder {
        hub_graph: &hub_graph,
    };
    let path_finder = SlowShortcutReplacer::new(&_shortcuts, &path_finder);

    test_cases
        .par_iter()
        .take(1_000)
        .progress()
        .for_each(|test_case| {
            let path = path_finder.shortest_path(&test_case.request);

            if let Err(err) = validate_path(&graph, test_case, &path) {
                panic!("top down hl wrong: {}", err);
            }
        });

    let writer = BufWriter::new(File::create("hl_test.bincode").unwrap());
    bincode::serialize_into(writer, &hub_graph).unwrap();

    test_cases
        .par_iter()
        .take(1_000)
        .progress()
        .for_each(|test_case| {
            // let forward_label = get_out_label(test_case.request.source(), &graph,
            // &order).0; let reverse_label =
            // get_in_label(test_case.request.target(), &graph, &order).0;
            // let (weight, _, _) = HubGraph::overlap(&forward_label,
            // &reverse_label).unwrap(); let weight = Some(weight);

            let weight = path_finder.shortest_path_weight(&test_case.request);
            assert_eq!(weight, test_case.weight);

            // let _path = hub_graph.shortest_path(&test_case.request);

            // if let Err(err) = validate_path(&graph, test_case, &_path) {
            //     panic!("top down hl wrong: {}", err);
            // }
        });

    println!("all {} tests passed", test_cases.len());
}

fn predict_average_label_size(
    test_cases: &Vec<ShortestPathTestCase>,
    graph: &dyn Graph,
    order: &Vec<u32>,
) -> f64 {
    let labels: Vec<_> = test_cases
        .par_iter()
        .take(1_000)
        .progress()
        .map(|test_case| {
            let mut shortcuts = HashMap::new();

            let (forward_label, forward_shortcuts) =
                get_out_label(test_case.request.source(), graph, order);
            let (reverse_label, reverse_shortcuts) =
                get_in_label(test_case.request.target(), graph, order);

            shortcuts.extend(forward_shortcuts.iter().cloned());
            shortcuts.extend(
                reverse_shortcuts
                    .into_iter()
                    .map(|(x, y)| (x.reversed(), y)),
            );

            let mut path = shortest_path(&forward_label, &reverse_label);

            if let Some(ref mut path) = path {
                replace_shortcuts_slow(&mut path.vertices, &shortcuts);
            }

            if let Err(err) = validate_path(graph, test_case, &path) {
                panic!("top down hl wrong: {}", err);
            }

            vec![forward_label, reverse_label]
        })
        .flatten()
        .collect();

    let average_label_size =
        labels.iter().map(|l| l.entries.len()).sum::<usize>() as f64 / labels.len() as f64;
    average_label_size
}
