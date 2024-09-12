use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Edge, Graph},
    search::{alt::landmark::Landmarks, collections::dijkstra_data::Path, DistanceHeuristic},
    utility::{get_progressbar, read_bincode_with_spinnner, read_json_with_spinnner},
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    simple_graph: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("graph", &args.graph.as_path());
    let mut edges = HashMap::new();
    for edge in graph.out_graph().all_edges() {
        edges.insert((edge.tail, edge.head), edge.weight);
    }

    // Build graph
    let simple_graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("simple graph", &args.simple_graph.as_path());

    let landmarks = Landmarks::hitting_set(&graph, 1000, 10);

    let mut vertices = graph.out_graph().vertices().collect_vec();
    vertices.shuffle(&mut thread_rng());

    let pb = get_progressbar("gett edge diff", vertices.len() as u64);
    let edge_diffs = vertices
        .par_iter()
        .progress_with(pb)
        .map(|&vertex| {
            let mut new_edges = 0;
            for in_edge in graph.in_graph().edges(vertex) {
                for out_edge in graph.out_graph().edges(vertex) {
                    // let edge = Edge {
                    //     tail: in_edge.head,
                    //     head: out_edge.head,
                    // };
                    if edges.get(&(in_edge.head, out_edge.head)).is_none() {
                        // graph.get_weight(&edge).is_none() {
                        if in_edge.weight + out_edge.weight
                            < landmarks.upper_bound(in_edge.head, out_edge.head)
                        {
                            new_edges += 1
                        }
                    }
                }
            }

            new_edges
                - graph.in_graph().edges(vertex).len() as i32
                - graph.out_graph().edges(vertex).len() as i32
        })
        .collect::<Vec<_>>();
}
