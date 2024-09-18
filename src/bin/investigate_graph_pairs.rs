use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Distance, Graph, Vertex,
    },
    search::{
        alt::landmark::Landmarks, collections::dijkstra_data::Path, hl::hub_graph::HubGraph,
        DistanceHeuristic, PathFinding,
    },
    utility::{get_paths, level_to_vertex, read_bincode_with_spinnner, read_json_with_spinnner},
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    hl_graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    simple_graph_hl: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    paths: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    data: PathBuf,
}

fn main() {
    let args = Args::parse();

    let paths: Vec<Path> = read_json_with_spinnner("paths", &args.paths.as_path());

    // Build graph
    let graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("graph", &args.graph.as_path());

    // Build graph
    let graph_hl: HubGraph = read_bincode_with_spinnner("graph hl", &&args.hl_graph.as_path());

    // Build graph
    let simple_graph_hl: HubGraph =
        read_bincode_with_spinnner("simple graph hl", &args.simple_graph_hl.as_path());

    // check_if_upper_bound(&graph, &simple_graph);

    // Get paths and level_to_vertex
    let paths_landmarks = get_paths(
        &graph_hl,
        &graph.out_graph().non_trivial_vertices(),
        100_000,
    );
    let level_to_vertex: Vec<Vertex> =
        level_to_vertex(&paths_landmarks, graph.number_of_vertices());

    println!("Simple graph used as upper bound");
    let simple_graph_bound = simple_graph_bound(&simple_graph_hl, &paths);

    let max_num_landmarks = 100;
    let landmarks_bounds = (1..=max_num_landmarks)
        .map(|n| {
            let landmarks = Landmarks::new(
                &graph,
                &level_to_vertex.iter().rev().take(n).cloned().collect_vec(),
            );
            println!("Landmarks used as upper bound");
            let landmarks_bound = landmarks_bound(&graph, &landmarks, &paths);
            landmarks_bound
        })
        .collect_vec();

    let mut writer = BufWriter::new(File::create(args.data.as_path()).unwrap());
    writeln!(
        writer,
        "distance,hops,simple_graph_upper_bound,{:?}",
        (1..=max_num_landmarks).map(|n| n.to_string()).join(",")
    )
    .unwrap();
    for i in 0..paths.len() {
        writeln!(
            writer,
            "{},{},{},{}",
            paths[i].distance,
            paths[i].vertices.len(),
            simple_graph_bound[i],
            (1..=max_num_landmarks)
                .map(|n| landmarks_bounds[n][i].to_string())
                .join(",")
        )
        .unwrap();
    }
    writer.flush().unwrap();
}

fn _check_if_upper_bound(
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

fn simple_graph_bound(simple_graph_hl: &dyn PathFinding, paths: &Vec<Path>) -> Vec<Distance> {
    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let simple_shortest_path = simple_graph_hl.shortest_path(source, target).unwrap();
            let upper_bound = simple_shortest_path.distance;

            assert!(upper_bound >= shortest_path.distance,);

            upper_bound
        })
        .collect::<Vec<_>>();

    print_results(&distance_pairs, paths);

    distance_pairs
}

fn landmarks_bound(
    graph: &ReversibleGraph<VecVecGraph>,
    landmarks: &Landmarks,
    paths: &Vec<Path>,
) -> Vec<Distance> {
    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let uppwer_bound = landmarks.upper_bound(source, target);

            assert!(uppwer_bound >= shortest_path.distance,);

            uppwer_bound
        })
        .collect::<Vec<_>>();

    print_results(&distance_pairs, paths);

    distance_pairs
}

fn simple_graph_and_landmarks_bound(
    graph: &ReversibleGraph<VecVecGraph>,
    simple_graph_hl: &dyn PathFinding,
    landmarks: &Landmarks,
    paths: &Vec<Path>,
) {
    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let simple_graph_upper_bound = simple_graph_hl
                .shortest_path_distance(source, target)
                .unwrap_or(Distance::MAX);
            let landmark_upper_bound = landmarks.upper_bound(source, target);

            let min_upper_bound = std::cmp::min(simple_graph_upper_bound, landmark_upper_bound);

            assert!(min_upper_bound >= shortest_path.distance);

            min_upper_bound
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
