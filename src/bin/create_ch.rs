use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant, usize};

use clap::Parser;
use faster_paths::{
    ch::contraction_non_adaptive::contract_non_adaptive,
    classical_search::{
        cache_dijkstra::CacheDijkstra,
        dijkstra::{self, Dijkstra},
    },
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::{
            all_edges, generate_hiting_set_order_with_hub_labels, generate_random_pair_testcases,
            hitting_set, random_paths, shortests_path_tree,
        },
        path::Path,
        Graph,
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};
use indicatif::{ParallelProgressIterator, ProgressIterator};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .gr or .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Outfile in .bincode format
    #[arg(short, long)]
    contracted_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Loading graph");
    let graph = GraphFactory::from_file(&args.graph);

    let n = 1000;
    let dijkstra = Dijkstra::new(&graph);
    let start = Instant::now();
    let paths = random_paths(n, graph.number_of_vertices(), &dijkstra);
    println!(
        "avg path len {}. took {:?} path path",
        paths.iter().map(|path| path.vertices.len()).sum::<usize>(),
        start.elapsed() / n
    );
    let hitting_set = hitting_set(&paths, graph.number_of_vertices()).0;
    println!("hitting set len {}", hitting_set.len());

    println!(
        "{:?}",
        all_edges(&graph)
            .iter()
            .max_by_key(|edge| edge.weight())
            .unwrap()
    );
    let mut cache_dijkstra = CacheDijkstra::new(&graph);
    cache_dijkstra.cache = hitting_set
        .par_iter()
        .progress()
        .map(|&vertex| {
            let data = cache_dijkstra.single_source(vertex);
            let tree = shortests_path_tree(&data);
            let data = data.vertices;
            (vertex, (data, tree))
        })
        .collect();

    let start = Instant::now();
    let paths = random_paths(n, graph.number_of_vertices(), &cache_dijkstra);
    println!(
        "avg path len {}. took {:?} path path",
        paths.iter().map(|path| path.vertices.len()).sum::<usize>(),
        start.elapsed() / n
    );

    let test_cases = generate_random_pair_testcases(10_000, &graph);

    let n = 10;
    let landmarks_avoid = Landmarks::avoid(n, &graph);
    let landmarks_random = Landmarks::new(n, &graph);

    for i in 1..n {
        let landmarks_avoid = Landmarks {
            landmarks: landmarks_avoid.landmarks[1..=i as usize].to_vec(),
        };

        let landmarks_random = Landmarks {
            landmarks: landmarks_random.landmarks[1..=i as usize].to_vec(),
        };

        let avg_avoid = test_cases
            .iter()
            .map(|test_case| {
                let lower_bound = landmarks_avoid.lower_bound(&test_case.request).unwrap_or(0);
                test_case
                    .weight
                    .unwrap_or(0)
                    .checked_sub(lower_bound)
                    .unwrap() as u64
            })
            .sum::<u64>();

        let avg_random = test_cases
            .iter()
            .map(|test_case| {
                let lower_bound = landmarks_random
                    .lower_bound(&test_case.request)
                    .unwrap_or(0);
                test_case
                    .weight
                    .unwrap_or(0)
                    .checked_sub(lower_bound)
                    .unwrap() as u64
            })
            .sum::<u64>();

        println!("{} {} {}", i, avg_avoid, avg_random);
    }

    println!("Starting contracted graph generation");
    let start = Instant::now();

    let number_of_random_pairs = 4_000;
    let mut order = generate_hiting_set_order_with_hub_labels(number_of_random_pairs, &graph);
    let order_copy = order.clone();
    order.sort_unstable_by_key(|v| order_copy.iter().position(|vv| vv == v).unwrap());

    let contracted_graph = contract_non_adaptive(&graph, &order);

    // let contracted_graph = contract_adaptive_simulated_with_witness(&graph);
    println!("Generating contracted graph took {:?}", start.elapsed());

    println!("Writing contracted graph to file");
    let writer = BufWriter::new(File::create(args.contracted_graph).unwrap());
    serde_json::to_writer(writer, &contracted_graph).unwrap();
}
