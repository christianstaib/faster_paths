use std::{fs::File, io::BufReader, path::PathBuf, sync::atomic::AtomicU32};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{
        alt::landmark::Landmarks,
        ch::{
            contracted_graph::ContractedGraph, contraction::edge_difference,
            probabilistic_contraction::par_simulate_contraction_distance_heuristic,
        },
        PathFinding,
    },
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
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
            let total_failed = total_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;

            println!(
                "{} -> {} failed. ({}% failed)",
                source,
                target,
                total_failed as f32 / all as f32 * 100.0
            );
        }
    })

    // // let heuristic = TrivialHeuristic {};
    // let heuristic = Landmarks::random(&graph, rayon::current_num_threads() as
    // u32 * 2);

    // let vertices = graph.out_graph().vertices().collect_vec();

    // for &vertex in vertices.choose_multiple(&mut thread_rng(), 100) {
    //     // let new_and_updated_edges_witness =
    //     //     par_simulate_contraction_witness_search(&graph, u32::MAX,
    // vertex);     // let edge_differece_witness =
    //     //     edge_difference(&graph, &new_and_updated_edges_witness,
    // vertex);     // println!(
    //     //     "edge_difference for vertex {} is {} (outgoing edges {})",
    //     //     vertex,
    //     //     edge_differece_witness,
    //     //     graph.out_graph().edges(vertex).len()
    //     // );

    //     let new_and_updated_edges_heuristic =
    //         par_simulate_contraction_distance_heuristic(&graph, &heuristic,
    // vertex);     let edge_differece_heuristic =
    //         edge_difference(&graph, &new_and_updated_edges_heuristic,
    // vertex);     println!(
    //         "edge_difference for vertex {} is {} (outgoing edges {})",
    //         vertex,
    //         edge_differece_heuristic,
    //         graph.out_graph().edges(vertex).len()
    //     );
    // }
}
