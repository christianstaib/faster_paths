use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::Parser;
use faster_paths::{
    graphs::{
        read_edges_from_fmi_file, reversible_graph::ReversibleGraph, vec_vec_graph::VecVecGraph,
        Graph, Vertex,
    },
    search::{
        ch::contracted_graph::vertex_to_level,
        hl::half_hub_graph::get_hub_label_with_brute_force_wrapped, PathFinding,
    },
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
    level_to_vertex: PathBuf,

    /// Infile in .fmi format
    #[arg(short, long)]
    part: u32,

    /// Infile in .fmi format
    #[arg(short, long)]
    step_size: u32,

    /// Infile in .fmi format
    #[arg(short, long)]
    dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    // Build graph
    let edges = read_edges_from_fmi_file(&args.graph);
    let graph = ReversibleGraph::<VecVecGraph>::from_edges(&edges);
    println!(
        "graph is bidirectional {}?",
        graph.out_graph().is_bidirectional()
    );

    //
    // Read vertex_to_level
    let reader = BufReader::new(File::open(&args.level_to_vertex).unwrap());
    let level_to_vertex: Vec<Vertex> = serde_json::from_reader(reader).unwrap();
    let vertex_to_level = vertex_to_level(&level_to_vertex);

    let labels_and_shortcuts = ((args.step_size * args.part)..(args.step_size * (args.part + 1)))
        .into_par_iter()
        .progress()
        .filter(|&vertex| vertex < graph.number_of_vertices())
        .map(|vertex| {
            (
                vertex,
                get_hub_label_with_brute_force_wrapped(graph.out_graph(), &vertex_to_level, vertex),
            )
        })
        .collect::<Vec<_>>();

    let mut all_labels = HashMap::new();
    let mut all_shortcuts = HashMap::new();

    for (vertex, (label, shortcuts)) in labels_and_shortcuts.into_iter() {
        all_labels.insert(vertex, label);
        all_shortcuts.extend(shortcuts);
    }

    let data = (all_labels, all_shortcuts);
    let writer = BufWriter::new(
        File::create(
            args.dir
                .join(format!("{}_{}.bincode", args.step_size, args.part)),
        )
        .unwrap(),
    );
    bincode::serialize_into(writer, &data).unwrap();
}
