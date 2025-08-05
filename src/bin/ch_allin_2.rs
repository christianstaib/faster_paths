use std::path::PathBuf;

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Distance, Graph, Vertex,
    },
    search::{
        alt::landmark::Landmarks, collections::dijkstra_data::Path, DistanceHeuristic, PathFinding,
    },
    utility::{level_to_vertex, read_bincode_with_spinnner, read_json_with_spinnner},
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    paths: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    simple_graph: PathBuf,
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

    println!(
        "true {}, simple {}",
        graph.shortest_path_distance(123, 2345).unwrap(),
        simple_graph.shortest_path_distance(123, 2345).unwrap()
    );

    println!("Simple graph used as upper bound");
    research_diff(&simple_graph, &paths);

    println!("Landmarks used as upper bound");
    research_landmarks(&graph, &paths);

    println!("Min of both used as upper bound");
    research_diff_landmarks_combined(&graph, &simple_graph, &paths);
}

fn research_landmarks(graph: &ReversibleGraph<VecVecGraph>, paths: &Vec<Path>) {
    let path_vertices_only = paths.iter().map(|path| path.vertices.clone()).collect_vec();
    let level_to_vertex: Vec<Vertex> =
        level_to_vertex(&path_vertices_only, graph.number_of_vertices());
    let landmarks = Landmarks::new(
        &graph,
        &level_to_vertex.into_iter().rev().take(20).collect_vec(),
    );

    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let landmark_distance = landmarks.upper_bound(source, target);

            assert!(landmark_distance >= shortest_path.distance);

            (
                shortest_path.vertices.len(),
                shortest_path.distance,
                landmark_distance,
            )
        })
        .collect::<Vec<_>>();

    let mut diffs_per_hops = Vec::new();
    for (hops, true_distance, upper_bound_distance) in distance_pairs {
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
            diffs.iter().sum::<f32>() / diffs.len() as f32
        );
    }
    println!(
        "average diff is {:>.4}%",
        total_diffs.iter().sum::<f32>() / total_diffs.len() as f32
    );
}

fn research_diff(simple_graph: &ReversibleGraph<VecVecGraph>, paths: &Vec<Path>) {
    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let simple_graph_distance = simple_graph
                .shortest_path_distance(source, target)
                .unwrap_or(Distance::MAX);

            assert!(simple_graph_distance >= shortest_path.distance * 100);

            (
                shortest_path.vertices.len(),
                shortest_path.distance * 100,
                simple_graph_distance,
            )
        })
        .collect::<Vec<_>>();

    let mut diffs_per_hops = Vec::new();
    for (hops, true_distance, upper_bound_distance) in distance_pairs {
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
        "average diff is {:>.4}",
        ((total_diffs.iter().sum::<f32>() / total_diffs.len() as f32) - 1.0) * 100.0
    );
}

fn research_diff_landmarks_combined(
    graph: &ReversibleGraph<VecVecGraph>,
    simple_graph: &ReversibleGraph<VecVecGraph>,
    paths: &Vec<Path>,
) {
    let path_vertices_only = paths.iter().map(|path| path.vertices.clone()).collect_vec();
    let level_to_vertex: Vec<Vertex> =
        level_to_vertex(&path_vertices_only, graph.number_of_vertices());
    let landmarks = Landmarks::new(
        &graph,
        &level_to_vertex.into_iter().rev().take(20).collect_vec(),
    );

    let distance_pairs = paths
        .into_par_iter()
        .progress()
        .map(|shortest_path| {
            let source = *shortest_path.vertices.first().unwrap();
            let target = *shortest_path.vertices.last().unwrap();

            let simple_graph_distance = simple_graph
                .shortest_path_distance(source, target)
                .unwrap_or(Distance::MAX);
            let landmark_distance = landmarks.upper_bound(source, target) * 100;

            let min_upper_bound = std::cmp::min(simple_graph_distance, landmark_distance);

            assert!(min_upper_bound >= shortest_path.distance * 100);

            (
                shortest_path.vertices.len(),
                shortest_path.distance * 100,
                simple_graph_distance,
            )
        })
        .collect::<Vec<_>>();

    let mut diffs_per_hops = Vec::new();
    for (hops, true_distance, upper_bound_distance) in distance_pairs {
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
            diffs.iter().sum::<f32>() / diffs.len() as f32
        );
    }
    println!(
        "average diff is {:>.5}",
        total_diffs.iter().sum::<f32>() / total_diffs.len() as f32
    );
}
