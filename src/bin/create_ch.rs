use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant, usize};

use clap::Parser;
use faster_paths::{
    ch::contraction_non_adaptive::contract_non_adaptive,
    graphs::{
        graph_factory::GraphFactory,
        graph_functions::{
            generate_hiting_set_order_with_hub_labels, generate_random_pair_testcases,
        },
    },
    heuristics::{landmarks::Landmarks, Heuristic},
};
use indicatif::ProgressIterator;
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

    let test_cases = generate_random_pair_testcases(1_00, &graph);

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
                println!("xxx {:?} {}", test_case.weight, lower_bound);
                test_case
                    .weight
                    .unwrap_or(0)
                    .checked_sub(lower_bound)
                    .unwrap()
            })
            .sum::<u32>();

        let avg_random = test_cases
            .iter()
            .map(|test_case| {
                let lower_bound = landmarks_random
                    .lower_bound(&test_case.request)
                    .unwrap_or(0);
                test_case.weight.unwrap_or(0) - lower_bound
            })
            .sum::<u32>();

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
