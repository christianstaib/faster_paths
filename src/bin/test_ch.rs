use std::{fs::File, io::BufReader, path::PathBuf, sync::atomic::AtomicU32};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{
        ch::contracted_graph::ContractedGraph, dijkstra::dijkstra_one_to_one_wrapped, PathFinding,
    },
};
use indicatif::ParallelProgressIterator;
use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// Starts a routing service on localhost:3030/route
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,
    /// Infile in .fmi format
    #[arg(short, long)]
    ch_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    // Read contracted_graph
    let reader = BufReader::new(File::open(&args.ch_graph).unwrap());
    let contracted_graph: ContractedGraph = bincode::deserialize_from(reader).unwrap();

    let total_failed = AtomicU32::new(0);
    let all = AtomicU32::new(0);

    (0..100_000_000).into_par_iter().progress().for_each(|_| {
        let source = thread_rng().gen_range(graph.out_graph().vertices());
        let target = thread_rng().gen_range(graph.out_graph().vertices());

        let path = contracted_graph.shortest_path(source, target);
        let path_distance = path
            .as_ref()
            .and_then(|path| graph.out_graph().get_path_distance(&path.vertices));
        let distance = path.as_ref().map(|path| path.distance);

        let all = all.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        if distance != path_distance {
            let dijkstra_distance = dijkstra_one_to_one_wrapped(graph.out_graph(), source, target)
                .map(|path| path.distance);
            let total_failed = total_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

            println!(
                "{} -> {} failed. ({}% failed) (dijkstra==ch:{}, dijkstra==path:{})",
                source,
                target,
                total_failed as f32 / all as f32 * 100.0,
                dijkstra_distance == distance,
                dijkstra_distance == path_distance
            );
        }
    })
}
