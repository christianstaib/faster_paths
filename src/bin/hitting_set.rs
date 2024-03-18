use std::{collections::BTreeSet, fs::File, io::BufReader, usize};

use clap::Parser;
use faster_paths::{
    ch::{
        shortcut_replacer::{fast_shortcut_replacer::FastShortcutReplacer, ShortcutReplacer},
        ContractedGraphInformation,
    },
    graphs::{
        fast_graph::FastGraph,
        graph_factory::GraphFactory,
        path::{PathFinding, ShortestPathRequest},
        VertexId,
    },
    hl::{hub_graph::HubGraph, hub_graph_path_finder::HubGraphPathFinder},
};
use indicatif::ProgressIterator;
use rand::Rng;
use rayon::iter::{ParallelBridge, ParallelIterator};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path of .fmi file
    #[arg(short, long)]
    graph_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    ch_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    hl_path: String,
    /// Path of .fmi file
    #[arg(short, long)]
    tests_path: String,
}

fn main() {
    let args = Args::parse();

    let slow_graph = GraphFactory::from_gr_file(args.graph_path.as_str());
    let fast_graph = FastGraph::from_graph(&slow_graph);

    let reader = BufReader::new(File::open(args.ch_path).unwrap());
    let ch_information: ContractedGraphInformation = bincode::deserialize_from(reader).unwrap();

    let fast_shortcut_replacer: Box<dyn ShortcutReplacer> =
        Box::new(FastShortcutReplacer::new(&ch_information.shortcuts));
    let reader = BufReader::new(File::open(args.hl_path).unwrap());
    let hl: HubGraph = bincode::deserialize_from(reader).unwrap();
    let ch = HubGraphPathFinder::new(hl, fast_shortcut_replacer);

    let mut hittings_set = BTreeSet::new();

    let n = 1_000;
    for _ in 0..1_000 {
        let mut hits = vec![0; fast_graph.number_of_vertices() as usize];
        let paths = get_paths(&fast_graph, &ch, n);
        let mut legal_paths = Vec::new();
        for path in paths.into_iter() {
            let mut legal = true;
            for v in path.iter() {
                if hittings_set.contains(v) {
                    legal = false;
                    break;
                }
            }
            if legal {
                legal_paths.push(path);
            }
        }

        legal_paths
            .iter()
            .flatten()
            .for_each(|&v| hits[v as usize] += 1);

        let max = (0..hits.len()).max_by_key(|&i| hits[i]).unwrap();
        println!(
            "selected {} who hits {}%",
            max,
            100.0 * hits[max] as f64 / n as f64
        );
        hittings_set.insert(max as VertexId);
    }

    let final_paths = get_paths(&fast_graph, &ch, 10 * n);
    let hitted_final_paths: Vec<_> = final_paths
        .iter()
        .filter(|path| path.iter().any(|v| hittings_set.contains(v)))
        .cloned()
        .collect();

    println!(
        "hitted {:>.2}%",
        100.0 * hitted_final_paths.len() as f64 / final_paths.len() as f64
    );
}

fn get_paths(fast_graph: &FastGraph, ch: &dyn PathFinding, n: u32) -> Vec<Vec<VertexId>> {
    (0..(n) as usize)
        .progress()
        .par_bridge()
        .map_init(
            rand::thread_rng, // get the thread-local RNG
            |rng, _| {
                // guarantee that source != tatget.
                let source = rng.gen_range(0..fast_graph.number_of_vertices());
                let mut target = rng.gen_range(0..fast_graph.number_of_vertices() - 1);
                if target >= source {
                    target += 1;
                }

                let request = ShortestPathRequest::new(source, target).unwrap();

                ch.get_shortest_path(&request)
            },
        )
        .filter_map(|path| path)
        .map(|path| path.vertices)
        .collect()
}
