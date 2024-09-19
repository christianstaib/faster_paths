use std::{cmp::Reverse, path::PathBuf};

use clap::Parser;
use faster_paths::{
    graphs::{reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph, Graph, Vertex},
    search::{ch::contracted_graph::vertex_to_level, hl::hub_graph::HubGraph, PathFinding},
    utility::{
        average_ch_vertex_degree, average_hl_label_size, get_paths, level_to_vertex,
        level_to_vertex_with_ord, read_bincode_with_spinnner,
    },
};
use rand::prelude::*;

// Predict average label size by brute forcing a number of labels.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Infile in .fmi format
    #[arg(short, long)]
    graph: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    hub_graph: PathBuf,

    /// Number of labels to calculate
    #[arg(short, long)]
    labels: u32,

    /// Infile in .fmi format
    #[arg(short, long)]
    simple_hub_graph: Option<PathBuf>,

    /// Number of labels to calculate
    #[arg(short, long)]
    paths: u32,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let graph = ReversibleGraph::<VecVecGraph>::from_fmi_file(&args.graph);

    let hub_graph: HubGraph = read_bincode_with_spinnner("hub graph", &args.hub_graph.as_path());

    // Get paths and level_to_vertex
    let paths = get_paths(
        &hub_graph,
        &graph.out_graph().non_trivial_vertices(),
        args.paths,
        3,
        usize::MAX,
    );

    let mut orderings = Vec::new();

    if let Some(simple_hub_graph) = args.simple_hub_graph {
        let simple_hub_graph: HubGraph =
            read_bincode_with_spinnner("hub graph", &simple_hub_graph.as_path());
        let level_to_vertex: Vec<Vertex> =
            level_to_vertex(&paths, simple_hub_graph.number_of_vertices());
        orderings.push(("simple ordering", level_to_vertex.clone()));
    }

    let level_to_vertex: Vec<Vertex> = level_to_vertex(&paths, hub_graph.number_of_vertices());
    orderings.push(("hitting-set, then hits", level_to_vertex.clone()));

    let level_to_vertex: Vec<Vertex> =
        level_to_vertex_with_ord(&paths, hub_graph.number_of_vertices(), false, |_vertex| {
            let mut rng = thread_rng();
            rng.gen::<u32>()
        });
    orderings.push(("hitting-set, then random", level_to_vertex.clone()));

    let level_to_vertex: Vec<Vertex> =
        level_to_vertex_with_ord(&paths, hub_graph.number_of_vertices(), false, |&vertex| {
            graph.out_graph().edges(vertex).len()
        });
    orderings.push((
        "hitting-set, then degree (small to large)",
        level_to_vertex.clone(),
    ));

    let level_to_vertex: Vec<Vertex> =
        level_to_vertex_with_ord(&paths, hub_graph.number_of_vertices(), true, |&vertex| {
            graph.out_graph().edges(vertex).len()
        });
    orderings.push((
        "hitting-set, then degree (small to large), then hits",
        level_to_vertex.clone(),
    ));

    let mut level_to_vertex: Vec<Vertex> =
        level_to_vertex_with_ord(&paths, hub_graph.number_of_vertices(), false, |&vertex| {
            Reverse(graph.out_graph().edges(vertex).len())
        });
    orderings.push((
        "hitting-set, then degree (large to small)",
        level_to_vertex.clone(),
    ));

    level_to_vertex.shuffle(&mut thread_rng());
    orderings.push(("random", level_to_vertex.clone()));

    level_to_vertex.sort_by_key(|&vertex| graph.out_graph().edges(vertex).len());
    orderings.push(("degree (small to large)", level_to_vertex.clone()));

    level_to_vertex.shuffle(&mut thread_rng());
    level_to_vertex.sort_by_key(|&vertex| Reverse(graph.out_graph().edges(vertex).len()));
    orderings.push(("degree (large to small)", level_to_vertex.clone()));

    println!("hl:");
    for (name, level_to_vertex) in orderings.iter() {
        let average_hl_label_size = average_hl_label_size(
            graph.out_graph(),
            &vertex_to_level(&level_to_vertex),
            args.labels,
        );
        println!("{:<70} {:>6.2} ", name, average_hl_label_size);
    }

    println!("\nch:");
    for (name, level_to_vertex) in orderings.iter() {
        let average_ch_vertex_degree = average_ch_vertex_degree(
            graph.out_graph(),
            &vertex_to_level(&level_to_vertex),
            args.labels,
        );
        println!("{:<70} {:>6.2} ", name, average_ch_vertex_degree,);
    }
}
