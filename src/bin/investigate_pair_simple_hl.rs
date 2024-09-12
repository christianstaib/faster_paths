use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Distance, Edge, Graph,
    },
    search::{
        alt::landmark::Landmarks, collections::dijkstra_data::Path, hl::hub_graph::HubGraph,
        DistanceHeuristic,
    },
    utility::{
        get_progressbar, read_bincode_with_spinnner, read_json_with_spinnner,
        write_json_with_spinnner,
    },
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

    /// Infile in .fmi format
    #[arg(short, long)]
    edge_diff: PathBuf,
}

fn main() {
    let args = Args::parse();

    let graph: ReversibleGraph<VecVecGraph> =
        read_bincode_with_spinnner("graph", &args.graph.as_path());

    let simple_graph_hl: HubGraph =
        read_bincode_with_spinnner("simple hub graph", &args.simple_graph_hl);

    println!(
        "{} comparisions needed",
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

    let vertices = graph.out_graph().vertices().collect_vec();

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
                    let alterntive_weight = in_edge.weight + out_edge.weight;
                    if alterntive_weight <= simple_graph_hl.upper_bound(in_edge.head, out_edge.head)
                    {
                        if alterntive_weight
                            <= *edges
                                .get(&(in_edge.head, out_edge.head))
                                .unwrap_or(&Distance::MAX)
                        {
                            new_edges += 1
                        }
                    }
                }
            }

            let diff = new_edges
                - graph.in_graph().edges(vertex).len() as i32
                - graph.out_graph().edges(vertex).len() as i32;

            diff
        })
        .collect::<Vec<_>>();

    write_json_with_spinnner("edge differences", &args.edge_diff, &edge_diffs);
}
