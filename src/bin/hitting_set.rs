use std::{
    collections::BTreeSet,
    fs::File,
    io::BufReader,
    sync::atomic::{AtomicU32, Ordering},
    usize,
};

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
use rayon::prelude::*;

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
    let hl_path_finder = HubGraphPathFinder::new(hl, fast_shortcut_replacer);

    let mut hittings_set = BTreeSet::new();

    let mut paths = get_paths(fast_graph.number_of_vertices(), &hl_path_finder, 10_000_000);

    let n = 100_000;
    for _ in 0..1_000 {
        println!("1");
        paths = paths
            .into_par_iter()
            .filter(|path| !path.iter().any(|v| hittings_set.contains(v)))
            .collect();

        println!("2");
        let mut hits: Vec<_> = (0..fast_graph.number_of_vertices())
            .map(|_| AtomicU32::new(0))
            .collect();
        for path in paths.into_iter() {
            for &v in path.iter() {
                hits[v as usize].fetch_add(1, Ordering::Relaxed);
            }
        }

        let max = (0..hits.len())
            .max_by_key(|&i| hits[i].load(Ordering::Release))
            .unwrap();

        println!("3");
        print!("max is {}", max);
        hittings_set.insert(max as VertexId);
    }

    let final_paths = get_paths(fast_graph.number_of_vertices(), &hl_path_finder, 10 * n);
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

fn get_paths(
    number_of_vertices: u32,
    ch: &dyn PathFinding,
    number_of_paths: u32,
) -> Vec<Vec<VertexId>> {
    (0..(number_of_paths) as usize)
        .progress()
        .par_bridge()
        .map_init(rand::thread_rng, |rng, _| {
            let source = rng.gen_range(0..number_of_vertices);
            let mut target = rng.gen_range(0..number_of_vertices - 1); // guarantee that source != tatget.
            if target >= source {
                target += 1;
            }

            let request = ShortestPathRequest::new(source, target).unwrap();

            ch.get_shortest_path(&request)
        })
        .flatten()
        .map(|path| path.vertices)
        .collect()
}
