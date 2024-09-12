use std::{mem::offset_of, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Distance, Graph, Vertex,
    },
    search::{
        alt::landmark::{self, Landmarks},
        collections::dijkstra_data::Path,
        DistanceHeuristic, PathFinding,
    },
    utility::{read_bincode_with_spinnner, read_json_with_spinnner},
};
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    simple_graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    paths: PathBuf,
}

fn main() {
    let args = Args::parse();

    let paths: Vec<Path> = read_json_with_spinnner("paths", &args.paths.as_path());

    // Build graph
    let graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("graph", &args.graph.as_path());

    // Build graph
    let simple_graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("simple graph", &args.simple_graph.as_path());

    check_if_upper_bound(&graph, &simple_graph);

    println!("Simple graph used as upper bound");
    simple_graph_bound(&simple_graph, &paths);

    println!("Landmarks used as upper bound");
    landmarks_bound(&graph, &paths);

    println!("Min of both used as upper bound");
    simple_graph_and_landmarks_bound(&graph, &simple_graph, &paths);
}

fn check_if_upper_bound(
    graph: &ReversibleGraph<VecVecGraph>,
    simple_graph: &ReversibleGraph<VecVecGraph>,
) {
    for edge in graph.out_graph().all_edges() {
        if let Some(simple_edge_weight) = simple_graph.out_graph().get_weight(&edge.remove_weight())
        {
            if simple_edge_weight < edge.weight {
                println!(
                    "{} -> {} has graph weight {} but simple graph weight {}",
                    edge.tail, edge.head, edge.weight, simple_edge_weight
                );
            }
        }
    }
}

fn simple_graph_bound(simple_graph: &ReversibleGraph<VecVecGraph>, paths: &Vec<Path>) {
    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let simple_shortest_path = simple_graph.shortest_path(source, target).unwrap();
            let simple_graph_distance = simple_shortest_path.distance;

            assert!(simple_graph_distance >= shortest_path.distance,);

            simple_graph_distance
        })
        .collect::<Vec<_>>();

    print_results(&distance_pairs, paths);
}

fn landmarks_bound(graph: &ReversibleGraph<VecVecGraph>, paths: &Vec<Path>) {
    let landmarks = Landmarks::random(&graph, 100);

    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let landmark_distance = landmarks.upper_bound(source, target);

            assert!(landmark_distance >= shortest_path.distance,);

            landmark_distance
        })
        .collect::<Vec<_>>();

    print_results(&distance_pairs, paths);
}

fn simple_graph_and_landmarks_bound(
    graph: &ReversibleGraph<VecVecGraph>,
    simple_graph: &ReversibleGraph<VecVecGraph>,
    paths: &Vec<Path>,
) {
    let landmarks = Landmarks::random(&graph, 100);

    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let simple_graph_distance = simple_graph
                .shortest_path_distance(source, target)
                .unwrap_or(Distance::MAX);
            let landmark_distance = landmarks.upper_bound(source, target);

            let min_upper_bound = std::cmp::min(simple_graph_distance, landmark_distance);

            assert!(min_upper_bound >= shortest_path.distance);

            simple_graph_distance
        })
        .collect::<Vec<_>>();

    print_results(&distance_pairs, paths);
}

fn print_results(distance_pairs: &Vec<u32>, paths: &Vec<Path>) {
    let mut diffs_per_hops = Vec::new();
    for (&upper_bound_distance, path) in distance_pairs.iter().zip(paths.iter()) {
        let hops = path.vertices.len();
        let true_distance = path.distance;
        if hops >= diffs_per_hops.len() {
            diffs_per_hops.resize(hops + 1, Vec::new());
        }
        diffs_per_hops
            .get_mut(hops)
            .unwrap()
            .push(upper_bound_distance as f32 / true_distance as f32);
    }

    let mut total_diffs = Vec::new();
    println!("diff per hops");
    for (hops, diffs) in diffs_per_hops.iter().enumerate() {
        total_diffs.extend(diffs.iter().cloned());
        println!(
            "{:>2} {:>.5}%",
            hops,
            ((diffs.iter().sum::<f32>() / diffs.len() as f32) - 1.0) * 100.0
        );
    }
    println!(
        "total diff is {:>.5}%",
        ((total_diffs.iter().sum::<f32>() / total_diffs.len() as f32) - 1.0) * 100.0
    );
}
