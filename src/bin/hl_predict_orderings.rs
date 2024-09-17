use std::{cmp::Reverse, path::PathBuf, time::Instant};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph,
    },
    search::{ch::contracted_graph::vertex_to_level, PathFinding},
    utility::{average_ch_vertex_degree, average_hl_label_size},
};
use itertools::Itertools;
use rand::prelude::*;

// Predict average label size by brute forcing a number of labels.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Number of labels to calculate
    #[arg(short, long)]
    num_labels: u32,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);

    let mut level_to_vertex = graph.out_graph().vertices().collect_vec();

    let mut orderings = Vec::new();

    level_to_vertex.shuffle(&mut thread_rng());
    orderings.push(("random", level_to_vertex.clone()));

    level_to_vertex.sort_by_key(|&vertex| graph.out_graph().edges(vertex).len());
    orderings.push(("degree (small to large)", level_to_vertex.clone()));

    level_to_vertex.shuffle(&mut thread_rng());
    level_to_vertex.sort_by_key(|&vertex| Reverse(graph.out_graph().edges(vertex).len()));
    orderings.push(("degree (large to small)", level_to_vertex.clone()));

    for (name, level_to_vertex) in orderings {
        let start = Instant::now();
        let average_hl_label_size = average_hl_label_size(
            graph.out_graph(),
            &vertex_to_level(&level_to_vertex),
            args.num_labels,
        );
        let duration = start.elapsed();
        println!(
            "{}: average hl label size will be approximately {:.1}. Full calculation will take {:?}",
            name, average_hl_label_size, duration / args.num_labels * graph.number_of_vertices()
        );

        let start = Instant::now();
        let average_ch_vertex_degree = average_ch_vertex_degree(
            graph.out_graph(),
            &vertex_to_level(&level_to_vertex),
            args.num_labels,
        );
        let duration = start.elapsed();
        println!(
            "{}: average ch edge degree will be approximately {:.1}. Full calculation will take {:?} ",
            name, average_ch_vertex_degree, duration / args.num_labels * graph.number_of_vertices()
        );
    }
}
