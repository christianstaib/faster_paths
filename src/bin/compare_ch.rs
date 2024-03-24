use itertools::Itertools;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    time::{Duration, Instant},
    vec,
};

use clap::Parser;
use faster_paths::{
    ch::{
        ch_path_finder::ChPathFinder,
        contractor::serial_contractor::SerialContractor,
        preprocessor::Preprocessor,
        shortcut_replacer::{slow_shortcut_replacer::SlowShortcutReplacer, ShortcutReplacer},
    },
    graphs::{graph_factory::GraphFactory, path::ShortestPathValidation},
};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
    /// Path of contracted_graph (output)
    #[arg(short, long)]
    ch_graph: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
}

fn generate_permutations(functions: Vec<&str>, values: Vec<&str>) -> Vec<String> {
    let all_combinations = (0..functions.len())
        .map(|_| values.iter())
        .multi_cartesian_product()
        .collect::<Vec<_>>();

    let mut permutations = Vec::new();
    for combination in all_combinations {
        // Skip combinations where all values are the same
        if combination[0] != &values[0] {
            if combination.iter().all(|&x| x == combination[0]) {
                continue;
            }
        }

        let permutation = functions
            .iter()
            .zip(combination.iter())
            .map(|(&function, &value)| format!("{}:{}", function, value))
            .collect::<Vec<_>>()
            .join("_");

        permutations.push(permutation);
    }

    permutations
}

fn main() {
    let args = Args::parse();

    let graph = GraphFactory::from_gr_file(args.graph_path.as_str());
    let reader = BufReader::new(File::open(args.tests_path.as_str()).unwrap());
    let tests: Vec<ShortestPathValidation> = serde_json::from_reader(reader).unwrap();

    let functions = vec!["E", "D", "C"];
    let values = vec!["1", "2", "3"];

    let letters = generate_permutations(functions, values);
    let letters: Vec<_> = letters.iter().map(|s| s.as_str()).collect();

    for f in letters.iter() {
        println!("{}", f);
    }

    for letters in letters {
        let contractor = Box::new(SerialContractor::new(letters));
        let preprocessor = Preprocessor::with_contractor(contractor);

        let start = Instant::now();
        let contracted_graph = preprocessor.get_ch(&graph);
        let ch_time = start.elapsed();

        let shortcut_replacer: Box<dyn ShortcutReplacer + Sync + Send> =
            Box::new(SlowShortcutReplacer::new(&contracted_graph.shortcuts));

        let ch = ChPathFinder::new(contracted_graph.ch_graph.clone(), shortcut_replacer);
        let mut times = Vec::new();
        let mut search_space_size = Vec::new();
        for test in tests.iter().take(10_000) {
            // let _ = ch.get_shortest_path_weight(&test.request);

            let before = Instant::now();
            let (_, _, forward, backward) = ch.get_data(&test.request);
            times.push(before.elapsed());
            search_space_size.push(forward.dijkstra_rank() + backward.dijkstra_rank());
        }
        let query_time: Duration = (times.iter().sum::<Duration>()) / (times.len() as u32);
        println!(
            "{:<5} ch construction: {:>9} s {:?}, {} shortcuts added, {} avg expanded nodes",
            letters,
            ch_time.as_secs(),
            query_time,
            contracted_graph.shortcuts.len(),
            search_space_size.iter().sum::<u32>() / search_space_size.len() as u32
        );

        let writer = BufWriter::new(
            File::create(format!("{}.ch_{}.bincode", args.graph_path, letters)).unwrap(),
        );
        bincode::serialize_into(writer, &contracted_graph).unwrap();
    }
}
