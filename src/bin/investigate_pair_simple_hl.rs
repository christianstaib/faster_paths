use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Edge, Graph},
    search::{
        alt::landmark::Landmarks, collections::dijkstra_data::Path, hl::hub_graph::HubGraph,
        DistanceHeuristic,
    },
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
    simple_graph_hl: PathBuf,
}

fn main() {
    let args = Args::parse();

    let simple_graph_hl: HubGraph =
        read_bincode_with_spinnner("simple hub graph", &args.simple_graph_hl);

    // Build graph
    let graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("graph", &args.graph.as_path());

    println!(
        "{} comparisions",
        graph
            .out_graph()
            .vertices()
            .map(|vertex| graph.out_graph().edges(vertex).len()
                * graph.in_graph().edges(vertex).len())
            .sum::<usize>()
    );

    let mut edges = HashMap::new();
    for edge in graph.out_graph().all_edges() {
        edges.insert((edge.tail, edge.head), edge.weight);
    }

    let mut vertices = graph.out_graph().vertices().collect_vec();
    vertices.shuffle(&mut thread_rng());

    let pb = get_progressbar("gett edge diff", vertices.len() as u64);
    let edge_diffs = vertices
        .par_iter()
        .progress_with(pb)
        .map(|&vertex| {
            let mut new_edges = 0;
            let in_edges = graph.in_graph().edges(vertex).collect_vec();
            let out_edges = graph.out_graph().edges(vertex).collect_vec();

            for in_edge in in_edges.iter() {
                for out_edge in out_edges.iter() {
                    // let edge = Edge {
                    //     tail: in_edge.head,
                    //     head: out_edge.head,
                    // };
                    if edges.get(&(in_edge.head, out_edge.head)).is_none() {
                        // graph.get_weight(&edge).is_none() {
                        if true
                        // in_edge.weight + out_edge.weight
                        //  < simple_graph_hl.upper_bound(in_edge.head, out_edge.head)
                        {
                            new_edges += 1
                        }
                    }
                }
            }

            let diff = new_edges
                - graph.in_graph().edges(vertex).len() as i32
                - graph.out_graph().edges(vertex).len() as i32;

            println!("{}", diff);
            diff
        })
        .collect::<Vec<_>>();
}
