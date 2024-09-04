use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph, Vertex,
    },
    search::{
        ch::contracted_graph::vertex_to_level,
        hl::half_hub_graph::get_hub_label_with_brute_force_wrapped,
    },
    utility::get_progressbar,
};
use indicatif::ParallelProgressIterator;
use itertools::Itertools;
use rand::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

// Predict average label size by brute forcing a number of labels.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Where level_to_vertex list shall be stored.
    #[arg(short, long)]
    level_to_vertex: PathBuf,

    /// Number of labels to calculate
    #[arg(short, long)]
    num_labels: u32,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);
    //
    // Read vertex_to_level
    let reader = BufReader::new(File::open(&args.level_to_vertex).unwrap());
    let level_to_vertex: Vec<Vertex> = serde_json::from_reader(reader).unwrap();
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let vertices = graph.out_graph().non_trivial_vertices();

    let vertices = vertices
        .choose_multiple(&mut thread_rng(), args.num_labels as usize)
        .cloned()
        .collect_vec();

    let labels = vertices
        .par_iter()
        .progress_with(get_progressbar("Getting labels", vertices.len() as u64))
        .map(|&vertex| {
            vec![
                get_hub_label_with_brute_force_wrapped(graph.out_graph(), &vertex_to_level, vertex)
                    .0,
                get_hub_label_with_brute_force_wrapped(graph.in_graph(), &vertex_to_level, vertex)
                    .0,
            ]
        })
        .flatten()
        .collect::<Vec<_>>();

    println!(
        "Average label size is {}",
        labels.iter().flatten().count() as f32 / labels.len() as f32
    );
}
